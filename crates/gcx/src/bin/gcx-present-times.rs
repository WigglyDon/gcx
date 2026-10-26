// SPDX-License-Identifier: MIT
//! Offline analysis of already admitted, app-scoped Android presentation history.
//! No ADB, device discovery, counter reset or application operation is performed.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Read};

const MAX_INPUT: u64 = 512 * 1024;
const PENDING: i64 = i64::MAX;
// A descriptive threshold, applied unchanged to controls and Home observations.
// It is not a game frame budget or a count of missed emulated frames.
const SLOW_NS: u64 = 40_000_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    snapshots: Vec<Snapshot>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    start_ns: u64,
    end_ns: u64,
    stats: String,
}

struct History {
    refresh_ns: u64,
    completed: BTreeSet<u64>,
    pending: usize,
    unresolved: usize,
}

#[derive(Serialize, Debug)]
struct Intervals {
    count: usize,
    min_ns: u64,
    p50_ns: u64,
    p95_ns: u64,
    max_ns: u64,
    completed_interval_rate_hz: f64,
    display_period_bins: [usize; 5],
    slower: SlowerIntervals,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Run {
    first_interval_1_based: usize,
    count: usize,
}

#[derive(Serialize, Debug)]
struct SlowerIntervals {
    strictly_greater_than_ns: u64,
    count: usize,
    fraction: f64,
    positions_1_based: Vec<usize>,
    end_offsets_from_baseline_ns: Vec<u64>,
    consecutive_runs: Vec<Run>,
    max_consecutive: usize,
    count_by_elapsed_third: [usize; 3],
    max_ends_in_one_second: usize,
    scope: &'static str,
}

#[derive(Serialize, Debug)]
struct Report {
    status: &'static str,
    snapshots: usize,
    observation_ns: u64,
    refresh_periods_ns: BTreeSet<u64>,
    history_continuous: bool,
    refresh_constant: bool,
    unique_new_presentations: usize,
    pending_records_observed: usize,
    unresolved_records_observed: usize,
    unchanged_observations: usize,
    first_progress_observed_by_ns: Option<u64>,
    trailing_no_progress_at_least_ns: u64,
    intervals: Option<Intervals>,
    scope: &'static str,
}

fn interval_stats(times: &[u64], refresh_ns: u64) -> Intervals {
    let elapsed = times.last().unwrap() - times[0];
    let intervals: Vec<_> = times.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let mut bins = [0; 5];
    let mut slower = SlowerIntervals {
        strictly_greater_than_ns: SLOW_NS,
        count: 0,
        fraction: 0.0,
        positions_1_based: Vec::new(),
        end_offsets_from_baseline_ns: Vec::new(),
        consecutive_runs: Vec::new(),
        max_consecutive: 0,
        count_by_elapsed_third: [0; 3],
        max_ends_in_one_second: 0,
        scope: "ordered interval endpoints; one-second windows are half-open; thirds use elapsed presentation time; positions/offsets stay private; not unique game frames or a scheduling cause",
    };
    for (index, &interval) in intervals.iter().enumerate() {
        // Bins: <1.5, [1.5,2.5), [2.5,3.5), [3.5,4.5), >=4.5
        // observed display periods. Wide integers avoid threshold overflow.
        let bin = [3_u128, 5, 7, 9]
            .iter()
            .position(|limit| u128::from(interval) * 2 < limit * u128::from(refresh_ns))
            .unwrap_or(4);
        bins[bin] += 1;
        if interval <= SLOW_NS {
            continue;
        }
        let position = index + 1;
        let offset = times[index + 1] - times[0];
        slower.positions_1_based.push(position);
        slower.end_offsets_from_baseline_ns.push(offset);
        let third = ((u128::from(offset) * 3 / u128::from(elapsed)) as usize).min(2);
        slower.count_by_elapsed_third[third] += 1;
        match slower.consecutive_runs.last_mut() {
            Some(run) if run.first_interval_1_based + run.count == position => run.count += 1,
            _ => slower.consecutive_runs.push(Run {
                first_interval_1_based: position,
                count: 1,
            }),
        }
    }
    slower.count = slower.positions_1_based.len();
    slower.fraction = slower.count as f64 / intervals.len() as f64;
    slower.max_consecutive = slower
        .consecutive_runs
        .iter()
        .map(|run| run.count)
        .max()
        .unwrap_or(0);
    let mut left = 0;
    for (right, &end) in slower.end_offsets_from_baseline_ns.iter().enumerate() {
        while end - slower.end_offsets_from_baseline_ns[left] >= 1_000_000_000 {
            left += 1;
        }
        slower.max_ends_in_one_second = slower.max_ends_in_one_second.max(right - left + 1);
    }
    let mut sorted = intervals.clone();
    sorted.sort_unstable();
    let percentile = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    Intervals {
        count: intervals.len(),
        min_ns: sorted[0],
        p50_ns: percentile(50),
        p95_ns: percentile(95),
        max_ns: *sorted.last().unwrap(),
        completed_interval_rate_hz: intervals.len() as f64 * 1e9 / elapsed as f64,
        display_period_bins: bins,
        slower,
    }
}

fn history(text: &str) -> Result<History, &'static str> {
    if text.len() > 16 * 1024 {
        return Err("history exceeds the bounded text limit");
    }
    let mut lines = text.lines();
    let refresh_ns: u64 = lines
        .next()
        .ok_or("missing refresh period")?
        .trim()
        .parse()
        .map_err(|_| "invalid refresh period")?;
    if !(1..=1_000_000_000).contains(&refresh_ns) {
        return Err("refresh period unavailable or outside the supported format");
    }
    let mut h = History {
        refresh_ns,
        completed: BTreeSet::new(),
        pending: 0,
        unresolved: 0,
    };
    let mut rows = 0;
    let mut previous = 0;
    for line in lines.filter(|line| !line.trim().is_empty()) {
        rows += 1;
        if rows > 127 {
            return Err("history exceeds the pinned Android13 record count");
        }
        let columns: Vec<_> = line.split_whitespace().collect();
        if columns.len() != 3 {
            return Err("expected desired, actual and ready timestamps");
        }
        let mut values = [0; 3];
        for (value, column) in values.iter_mut().zip(columns) {
            *value = column.parse::<i64>().map_err(|_| "invalid timestamp")?;
            if *value < -1 {
                return Err("unknown negative timestamp");
            }
        }
        // Android13 FrameTracker dumps desired / actual / ready, in that order.
        // Zero, invalid-fence (-1) and pending-fence (INT64_MAX) are not frames.
        let actual = values[1];
        if actual == PENDING {
            h.pending += 1;
        } else if actual <= 0 {
            h.unresolved += 1;
        } else {
            let actual = actual as u64;
            if actual < previous {
                return Err("presentation history went backwards");
            }
            previous = actual;
            h.completed.insert(actual);
        }
    }
    Ok(h)
}

fn analyze(observation: Observation) -> Result<Report, &'static str> {
    let n = observation.snapshots.len();
    if !(2..=32).contains(&n) {
        return Err("two to 32 snapshots are required");
    }
    let first = &observation.snapshots[0];
    let mut report = Report {
        status: "observed_intervals",
        snapshots: n,
        observation_ns: 0,
        refresh_periods_ns: BTreeSet::new(),
        history_continuous: true,
        refresh_constant: true,
        unique_new_presentations: 0,
        pending_records_observed: 0,
        unresolved_records_observed: 0,
        unchanged_observations: 0,
        first_progress_observed_by_ns: None,
        trailing_no_progress_at_least_ns: 0,
        intervals: None,
        scope: "completed surface intervals from the last baseline frame (which can predate the first read); initial/trailing staleness explicit; not unique game frames, emulation speed, audio or input latency",
    };
    let mut baseline = None;
    let mut previous_frame = None;
    let mut previous_end = first.start_ns;
    let mut last_progress_end = first.end_ns;
    let mut frames = BTreeSet::new();
    for (index, snapshot) in observation.snapshots.iter().enumerate() {
        if snapshot.start_ns < previous_end || snapshot.end_ns < snapshot.start_ns {
            return Err("unordered host observation intervals");
        }
        previous_end = snapshot.end_ns;
        let h = history(&snapshot.stats)?;
        report.refresh_periods_ns.insert(h.refresh_ns);
        report.pending_records_observed += h.pending;
        report.unresolved_records_observed += h.unresolved;
        let latest = h.completed.last().copied();
        if index == 0 {
            baseline = latest;
        } else if let Some(previous) = previous_frame {
            if latest.is_some_and(|now| now < previous) {
                return Err("surface history reset or changed between observations");
            }
            if !h.completed.contains(&previous) {
                report.status = "history_gap";
                report.history_continuous = false;
            }
            if latest == Some(previous) {
                report.unchanged_observations += 1;
            } else if latest.is_some_and(|now| now > previous) {
                last_progress_end = snapshot.end_ns;
                report
                    .first_progress_observed_by_ns
                    .get_or_insert(snapshot.end_ns.saturating_sub(first.end_ns));
            }
        }
        if let Some(baseline) = baseline {
            frames.extend(h.completed.range((baseline + 1)..).copied());
        }
        previous_frame = latest;
    }
    let last = observation.snapshots.last().unwrap();
    report.observation_ns = last.end_ns - first.start_ns;
    report.trailing_no_progress_at_least_ns = last.start_ns.saturating_sub(last_progress_end);
    report.unique_new_presentations = frames.len();
    report.refresh_constant = report.refresh_periods_ns.len() == 1;
    if let Some(baseline) = baseline {
        if report.status == "history_gap" {
            return Ok(report);
        }
        if !report.refresh_constant {
            report.status = "changed_refresh";
            return Ok(report);
        }
        report.status = match frames.len() {
            0 => "no_new_presentations",
            1 => "insufficient_new_presentations",
            _ => "observed_intervals",
        };
        if frames.len() >= 2 {
            let times: Vec<_> = std::iter::once(baseline).chain(frames).collect();
            report.intervals = Some(interval_stats(
                &times,
                *report.refresh_periods_ns.first().unwrap(),
            ));
        }
    } else {
        report.status = "no_baseline_history";
    }
    Ok(report)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: gcx-present-times observation.json")?;
    if args.next().is_some() {
        return Err("expected one offline observation file".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_INPUT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_INPUT {
        return Err("observation exceeds 512 KiB".into());
    }
    let report = analyze(serde_json::from_slice(&bytes)?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(index: u64, frames: &[u64]) -> Snapshot {
        Snapshot {
            start_ns: index * 1_000_000_000,
            end_ns: index * 1_000_000_000 + 100,
            stats: format!(
                "16666667\n{}",
                frames
                    .iter()
                    .map(|t| format!("{} {t} {}\n", t + 1000, t + 2000))
                    .collect::<String>()
            ),
        }
    }

    #[test]
    fn actual_column_and_overlapping_history_are_not_counted_twice() {
        let report = analyze(Observation {
            snapshots: vec![
                snapshot(0, &[10, 20]),
                snapshot(1, &[10, 20, 30, 40]),
                snapshot(2, &[20, 30, 40, 50]),
            ],
        })
        .unwrap();
        assert_eq!(report.unique_new_presentations, 3);
        assert_eq!(report.intervals.unwrap().max_ns, 10);
        assert_eq!(report.status, "observed_intervals");
    }

    #[test]
    fn pending_invalid_and_duplicate_presentations_are_explicit() {
        let h = history(&format!(
            "16666667\n0 0 0\n1 -1 -1\n2 {PENDING} {PENDING}\n3 40 45\n4 40 46\n"
        ))
        .unwrap();
        assert_eq!(h.completed.into_iter().collect::<Vec<_>>(), vec![40]);
        assert_eq!((h.pending, h.unresolved), (1, 2));
    }

    #[test]
    fn a_long_completed_interval_is_retained() {
        let report = analyze(Observation {
            snapshots: vec![snapshot(0, &[1]), snapshot(1, &[1, 2, 3, 1_000_000_003])],
        })
        .unwrap();
        assert_eq!(report.intervals.unwrap().max_ns, 1_000_000_000);
    }

    #[test]
    fn lost_overlap_never_manufactures_an_interval_rate() {
        let report = analyze(Observation {
            snapshots: vec![snapshot(0, &[1, 2]), snapshot(1, &[3, 4, 5])],
        })
        .unwrap();
        assert_eq!(report.status, "history_gap");
        assert!(report.intervals.is_none());
    }

    #[test]
    fn a_stale_tail_is_visible_even_when_completed_intervals_are_short() {
        let report = analyze(Observation {
            snapshots: vec![
                snapshot(0, &[1]),
                snapshot(1, &[1, 2, 3]),
                snapshot(2, &[1, 2, 3]),
                snapshot(3, &[1, 2, 3]),
            ],
        })
        .unwrap();
        assert_eq!(report.unchanged_observations, 2);
        assert_eq!(report.trailing_no_progress_at_least_ns, 1_999_999_900);
    }

    #[test]
    fn absent_or_nonadvancing_counters_are_not_zero_fps() {
        for frames in [vec![], vec![1, 2]] {
            let report = analyze(Observation {
                snapshots: vec![snapshot(0, &frames), snapshot(1, &frames)],
            })
            .unwrap();
            assert!(report.intervals.is_none());
            assert_ne!(report.status, "observed_intervals");
        }
    }

    #[test]
    fn malformed_regressing_and_excessive_data_refuse() {
        for text in [
            "permission denied",
            "0\n",
            "16666667\n1 2\n",
            "16666667\n1 -2 3\n",
            "16666667\n1 9 2\n2 8 3\n",
        ] {
            assert!(history(text).is_err(), "{text}");
        }
        assert!(history(&format!("16666667\n{}", "0 0 0\n".repeat(128))).is_err());
        assert!(
            analyze(Observation {
                snapshots: vec![snapshot(0, &[3]), snapshot(1, &[2])]
            })
            .is_err()
        );
        assert!(analyze(Observation { snapshots: vec![] }).is_err());
    }

    #[test]
    fn columns_with_different_intervals_use_actual_present_time() {
        let h = history("16666667\n900 10 1000\n10000 20 15000\n").unwrap();
        assert_eq!(h.completed.into_iter().collect::<Vec<_>>(), vec![10, 20]);
    }

    #[test]
    fn the_interval_from_baseline_to_first_new_frame_is_not_hidden() {
        let report = analyze(Observation {
            snapshots: vec![
                snapshot(0, &[1]),
                snapshot(1, &[1, 1_000_000_001, 1_000_000_002]),
            ],
        })
        .unwrap();
        let intervals = report.intervals.unwrap();
        assert_eq!(intervals.max_ns, 1_000_000_000);
        assert_eq!(intervals.slower.positions_1_based, vec![1]);
    }

    fn stats(deltas: &[u64]) -> Intervals {
        let mut times = vec![1];
        for delta in deltas {
            times.push(times.last().unwrap() + delta);
        }
        interval_stats(&times, 10)
    }

    #[test]
    fn slow_frequency_keeps_order_and_distinguishes_runs_from_temporal_clusters() {
        let slower = stats(&[50_000_000, 20_000_000, 50_000_000, 20_000_000, 50_000_000]).slower;
        assert_eq!(slower.count, 3);
        assert_eq!(slower.fraction, 0.6);
        assert_eq!(slower.positions_1_based, vec![1, 3, 5]);
        assert_eq!(
            slower.end_offsets_from_baseline_ns,
            vec![50_000_000, 120_000_000, 190_000_000]
        );
        assert_eq!(slower.max_consecutive, 1);
        assert_eq!(slower.consecutive_runs.len(), 3);
        assert_eq!(slower.count_by_elapsed_third, [1, 1, 1]);
        assert_eq!(slower.max_ends_in_one_second, 3);
    }

    #[test]
    fn strict_threshold_and_adjacent_runs_have_explicit_boundaries() {
        let slower = stats(&[SLOW_NS, SLOW_NS + 1, SLOW_NS + 2, 1, SLOW_NS + 3]).slower;
        assert_eq!(slower.positions_1_based, vec![2, 3, 5]);
        assert_eq!(slower.max_consecutive, 2);
        assert_eq!(
            slower.consecutive_runs,
            vec![
                Run {
                    first_interval_1_based: 2,
                    count: 2
                },
                Run {
                    first_interval_1_based: 5,
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn one_second_clustering_does_not_include_both_ends_of_a_half_open_window() {
        let slower = stats(&[50_000_000, 950_000_000, 50_000_000]).slower;
        assert_eq!(slower.count, 3);
        assert_eq!(slower.max_consecutive, 3);
        assert_eq!(slower.max_ends_in_one_second, 2);
    }

    #[test]
    fn display_period_bins_use_observed_refresh_and_exact_half_period_boundaries() {
        assert_eq!(
            stats(&[14, 15, 24, 25, 34, 35, 44, 45]).display_period_bins,
            [1, 2, 2, 2, 1]
        );
        assert_eq!(stats(&[14, 15, 24]).slower.count, 0);
    }

    #[test]
    fn refresh_changes_withhold_pooled_interval_and_slow_group_statistics() {
        let mut changed = snapshot(1, &[1, 2, 3, 4]);
        changed.stats = changed.stats.replacen("16666667", "11111111", 1);
        let report = analyze(Observation {
            snapshots: vec![snapshot(0, &[1, 2]), changed],
        })
        .unwrap();
        assert_eq!(report.status, "changed_refresh");
        assert!(report.history_continuous);
        assert!(!report.refresh_constant);
        assert!(report.intervals.is_none());
    }

    #[test]
    fn history_gap_and_refresh_change_remain_separately_visible() {
        let mut changed = snapshot(1, &[3, 4]);
        changed.stats = changed.stats.replacen("16666667", "11111111", 1);
        let report = analyze(Observation {
            snapshots: vec![snapshot(0, &[1, 2]), changed],
        })
        .unwrap();
        assert_eq!(report.status, "history_gap");
        assert!(!report.history_continuous);
        assert!(!report.refresh_constant);
        assert!(report.intervals.is_none());
    }
}
