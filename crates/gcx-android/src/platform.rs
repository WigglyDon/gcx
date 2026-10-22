// SPDX-License-Identifier: MIT
use android_activity::AndroidApp;
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    sys::jint,
};
use std::{
    collections::VecDeque,
    ffi::{CStr, c_char},
    fs::File,
    io::Seek,
    os::fd::{FromRawFd, IntoRawFd},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
static VM: OnceLock<JavaVM> = OnceLock::new();
static ACTIVITY: Mutex<Option<GlobalRef>> = Mutex::new(None);
static MENU: Mutex<Option<GlobalRef>> = Mutex::new(None);
static COMMANDS: Mutex<VecDeque<Command>> = Mutex::new(VecDeque::new());
static EPOCH: AtomicU64 = AtomicU64::new(0);

pub enum Command {
    NativeError {
        game_id: String,
        epoch: u64,
        message: String,
    },
    Choose,
    Restart,
    Selected(String),
    Canceled,
    MenuHidden,
    Error(String),
    Notice(String),
}
pub fn push(command: Command) {
    if let Ok(mut queue) = COMMANDS.lock() {
        if queue.len() >= 16 {
            // Preserve at least the first actionable failure through an alert burst.
            let redundant = match &command {
                Command::Error(_) => queue.iter().any(|c| matches!(c, Command::Error(_))),
                Command::NativeError { epoch, .. } => queue
                    .iter()
                    .any(|c| matches!(c,Command::NativeError {epoch:old,..} if old==epoch)),
                _ => true,
            };
            if redundant {
                return;
            }
            queue.pop_back();
        }
        queue.push_back(command);
    }
}
pub fn commands() -> Vec<Command> {
    COMMANDS
        .lock()
        .map(|mut q| q.drain(..).collect())
        .unwrap_or_default()
}
pub fn init(app: &AndroidApp) -> Result<()> {
    EPOCH.fetch_add(1, Ordering::AcqRel);
    COMMANDS
        .lock()
        .map_err(|_| "command lock poisoned")?
        .clear();
    // SAFETY: AndroidApp supplies the process VM and NativeActivity's global object.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let _ = VM.set(vm);
    let env = VM.get().ok_or("VM unavailable")?.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    *ACTIVITY.lock().map_err(|_| "activity lock poisoned")? = Some(env.new_global_ref(activity)?);
    Ok(())
}
fn with_env<T>(f: impl FnOnce(&mut JNIEnv<'_>, &JObject<'_>) -> Result<T>) -> Result<T> {
    let activity = ACTIVITY
        .lock()
        .map_err(|_| "activity lock poisoned")?
        .clone()
        .ok_or("activity unavailable")?;
    let mut env = VM.get().ok_or("VM unavailable")?.attach_current_thread()?;
    // Callers return owned Rust values or global references. Android's Java
    // main thread stays attached, so temporary JNI references need a frame.
    let result = env.with_local_frame(64, |env| f(env, activity.as_obj()));
    if env.exception_check()? {
        env.exception_clear()?;
    }
    result
}
pub fn shutdown() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
    if let Ok(mut menu) = MENU.lock() {
        *menu = None;
    }
    if let Ok(mut activity) = ACTIVITY.lock() {
        *activity = None;
    }
}
fn uri<'a>(env: &mut JNIEnv<'a>, text: &str) -> Result<JObject<'a>> {
    let value = env.new_string(text)?;
    Ok(env
        .call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&value)],
        )?
        .l()?)
}
fn resolver<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'_>) -> Result<JObject<'a>> {
    Ok(env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )?
        .l()?)
}
pub fn open_document(text: &str) -> Result<File> {
    with_env(|env, activity| {
        let uri = uri(env, text)?;
        let resolver = resolver(env, activity)?;
        let mode = env.new_string("r")?;
        let pfd = env
            .call_method(
                resolver,
                "openFileDescriptor",
                "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
                &[JValue::Object(&uri), JValue::Object(&mode)],
            )?
            .l()?;
        if pfd.is_null() {
            return Err("provider returned no file".into());
        }
        let fd = env.call_method(&pfd, "detachFd", "()I", &[])?.i()?;
        if fd < 0 {
            return Err("provider returned invalid descriptor".into());
        }
        // SAFETY: detachFd transfers exclusive ownership to us.
        let mut file = unsafe { File::from_raw_fd(fd) };
        file.stream_position()?;
        Ok(file)
    })
}
pub fn display_name(text: &str) -> Result<String> {
    with_env(|env, activity| {
        let uri = uri(env, text)?;
        let resolver = resolver(env, activity)?;
        let null = JObject::null();
        let cursor = env.call_method(resolver, "query", "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;", &[JValue::Object(&uri), JValue::Object(&null), JValue::Object(&null), JValue::Object(&null), JValue::Object(&null)])?.l()?;
        if cursor.is_null() {
            return Err("provider metadata unavailable".into());
        }
        let result = (|| {
            if !env.call_method(&cursor, "moveToFirst", "()Z", &[])?.z()? {
                return Err("provider metadata empty".into());
            }
            let column = env.new_string("_display_name")?;
            let index = env
                .call_method(
                    &cursor,
                    "getColumnIndex",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&column)],
                )?
                .i()?;
            if index < 0 {
                return Err("provider has no display name".into());
            }
            let value = JString::from(
                env.call_method(
                    &cursor,
                    "getString",
                    "(I)Ljava/lang/String;",
                    &[JValue::Int(index)],
                )?
                .l()?,
            );
            Ok(env.get_string(&value)?.into())
        })();
        let _ = env.call_method(cursor, "close", "()V", &[]);
        result
    })
}
pub fn picker(app: &AndroidApp) {
    let epoch = EPOCH.load(Ordering::Acquire);
    app.run_on_java_main_thread(Box::new(move || {
        if EPOCH.load(Ordering::Acquire) != epoch {
            return;
        }
        let result = with_env(|env, activity| {
            let action = env.new_string("android.intent.action.OPEN_DOCUMENT")?;
            let intent = env.new_object(
                "android/content/Intent",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&action)],
            )?;
            let category = env.new_string("android.intent.category.OPENABLE")?;
            env.call_method(
                &intent,
                "addCategory",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&category)],
            )?;
            let mime = env.new_string("*/*")?;
            env.call_method(
                &intent,
                "setType",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&mime)],
            )?;
            env.call_method(
                &intent,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[JValue::Int(1 | 64)],
            )?;
            env.call_method(
                activity,
                "startActivityForResult",
                "(Landroid/content/Intent;I)V",
                &[JValue::Object(&intent), JValue::Int(42)],
            )?;
            Ok(())
        });
        if result.is_err() {
            push(Command::Error("Could not open the document picker.".into()));
        }
    }));
}
pub fn menu(app: &AndroidApp, show: bool, message: Option<String>) {
    let epoch = EPOCH.load(Ordering::Acquire);
    app.run_on_java_main_thread(Box::new(move || {
        if EPOCH.load(Ordering::Acquire) != epoch {
            return;
        }
        let result = with_env(|env, activity| {
            if let Some(view) = MENU.lock().map_err(|_| "menu lock poisoned")?.take() {
                let parent = env
                    .call_method(
                        view.as_obj(),
                        "getParent",
                        "()Landroid/view/ViewParent;",
                        &[],
                    )?
                    .l()?;
                if !parent.is_null() {
                    env.call_method(
                        parent,
                        "removeView",
                        "(Landroid/view/View;)V",
                        &[JValue::Object(view.as_obj())],
                    )?;
                }
            }
            if show {
                let view = env.new_object(
                    "android/widget/LinearLayout",
                    "(Landroid/content/Context;)V",
                    &[JValue::Object(activity)],
                )?;
                env.call_method(&view, "setOrientation", "(I)V", &[JValue::Int(1)])?;
                env.call_method(&view, "setGravity", "(I)V", &[JValue::Int(17)])?;
                let label = env.new_object(
                    "android/widget/TextView",
                    "(Landroid/content/Context;)V",
                    &[JValue::Object(activity)],
                )?;
                let recovery = message.is_some();
                let text = env.new_string(
                    message.unwrap_or_else(|| "gcx\n\nChoose a GameCube game".into()),
                )?;
                env.call_method(
                    &label,
                    "setText",
                    "(Ljava/lang/CharSequence;)V",
                    &[JValue::Object(&text)],
                )?;
                env.call_method(&label, "setTextSize", "(F)V", &[JValue::Float(20.0)])?;
                env.call_method(&label, "setGravity", "(I)V", &[JValue::Int(17)])?;
                env.call_method(&label, "setTextColor", "(I)V", &[JValue::Int(-1)])?;
                env.call_method(
                    &view,
                    "addView",
                    "(Landroid/view/View;)V",
                    &[JValue::Object(&label)],
                )?;
                env.call_method(
                    &view,
                    "setBackgroundColor",
                    "(I)V",
                    &[JValue::Int(0xff14232b_u32 as i32)],
                )?;
                for (action, text) in [
                    (2, "Restart with current saves"),
                    (1, "Choose a GameCube game"),
                ] {
                    if action == 2 && !recovery {
                        continue;
                    }
                    let button = env.new_object(
                        "android/widget/Button",
                        "(Landroid/content/Context;)V",
                        &[JValue::Object(activity)],
                    )?;
                    let text = env.new_string(text)?;
                    env.call_method(
                        &button,
                        "setText",
                        "(Ljava/lang/CharSequence;)V",
                        &[JValue::Object(&text)],
                    )?;
                    let class = env
                        .call_method(activity, "getClass", "()Ljava/lang/Class;", &[])?
                        .l()?;
                    let loader = env
                        .call_method(class, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
                        .l()?;
                    let name = env.new_string("com.machinerhythm.gcx.GcxActivity$Action")?;
                    let class = JClass::from(
                        env.call_method(
                            loader,
                            "loadClass",
                            "(Ljava/lang/String;)Ljava/lang/Class;",
                            &[JValue::Object(&name)],
                        )?
                        .l()?,
                    );
                    let listener = env.new_object(
                        class,
                        "(Lcom/machinerhythm/gcx/GcxActivity;I)V",
                        &[JValue::Object(activity), JValue::Int(action)],
                    )?;
                    env.call_method(
                        &button,
                        "setOnClickListener",
                        "(Landroid/view/View$OnClickListener;)V",
                        &[JValue::Object(&listener)],
                    )?;
                    env.call_method(
                        &view,
                        "addView",
                        "(Landroid/view/View;)V",
                        &[JValue::Object(&button)],
                    )?;
                }
                let params = env.new_object(
                    "android/view/ViewGroup$LayoutParams",
                    "(II)V",
                    &[JValue::Int(-1), JValue::Int(-1)],
                )?;
                env.call_method(
                    activity,
                    "addContentView",
                    "(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V",
                    &[JValue::Object(&view), JValue::Object(&params)],
                )?;
                *MENU.lock().map_err(|_| "menu lock poisoned")? = Some(env.new_global_ref(view)?);
            }
            Ok(())
        });
        if result.is_err() {
            push(Command::Error(
                "Could not prepare the player surface.".into(),
            ));
        } else if !show {
            push(Command::MenuHidden);
        }
    }));
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_machinerhythm_gcx_GcxActivity_uiAction(
    env: JNIEnv<'_>,
    activity: JObject<'_>,
    action: jint,
) {
    if !current_activity(&env, &activity) {
        return;
    }
    match action {
        1 => push(Command::Choose),
        2 => push(Command::Restart),
        _ => {}
    }
}
pub fn notice(app: &AndroidApp, message: String) {
    let epoch = EPOCH.load(Ordering::Acquire);
    app.run_on_java_main_thread(Box::new(move || {
        if EPOCH.load(Ordering::Acquire) != epoch {
            return;
        }
        let _ = with_env(|env, activity| {
            let message = env.new_string(message)?;
            let toast = env
                .call_static_method(
                    "android/widget/Toast",
                    "makeText",
                    "(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;",
                    &[
                        JValue::Object(activity),
                        JValue::Object(&message),
                        JValue::Int(1),
                    ],
                )?
                .l()?;
            env.call_method(toast, "show", "()V", &[])?;
            Ok(())
        });
    }));
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_machinerhythm_gcx_GcxActivity_documentResult(
    mut env: JNIEnv<'_>,
    activity: JObject<'_>,
    request: jint,
    result: jint,
    intent: JObject<'_>,
) {
    if !current_activity(&env, &activity) {
        return;
    }
    if request != 42 {
        return;
    }
    if result != -1 || intent.is_null() {
        push(Command::Canceled);
        return;
    }
    let selected: Result<String> = (|| {
        let uri = env
            .call_method(intent, "getData", "()Landroid/net/Uri;", &[])?
            .l()?;
        if uri.is_null() {
            return Err("no document selected".into());
        }
        let value = JString::from(
            env.call_method(&uri, "toString", "()Ljava/lang/String;", &[])?
                .l()?,
        );
        let text = env.get_string(&value)?.into();
        with_env(|env, activity| {
            let resolver = resolver(env, activity)?;
            env.call_method(
                resolver,
                "takePersistableUriPermission",
                "(Landroid/net/Uri;I)V",
                &[JValue::Object(&uri), JValue::Int(1)],
            )?;
            Ok(())
        })?;
        Ok(text)
    })();
    match selected {
        Ok(uri) => push(Command::Selected(uri)),
        Err(_) => {
            let _ = env.exception_clear();
            push(Command::Error(
                "Could not retain read access to that document.".into(),
            ));
        }
    }
}

fn current_activity(env: &JNIEnv<'_>, activity: &JObject<'_>) -> bool {
    ACTIVITY
        .lock()
        .ok()
        .and_then(|a| a.clone())
        .is_some_and(|bound| {
            env.is_same_object(bound.as_obj(), activity)
                .unwrap_or(false)
        })
}

/// Called only by the mechanical native file adapter; no provider writes.
pub extern "C" fn dispatch(
    op: u32,
    path: *const c_char,
    argument: *const c_char,
    output: *mut c_char,
    capacity: usize,
) -> i64 {
    let result: Result<i64> = (|| {
        // SAFETY: native/bridge.h lends valid NUL-terminated strings for this call.
        let path = unsafe { CStr::from_ptr(path) }.to_str()?;
        let argument = unsafe { CStr::from_ptr(argument) }.to_str()?;
        match op {
            1 if argument == "r" => Ok(i64::from(open_document(path)?.into_raw_fd())),
            2 => Ok(0),
            3 => Ok(i64::try_from(open_document(path)?.metadata()?.len())?),
            4 => {
                let text = display_name(path)?;
                if !output.is_null() {
                    if text.len() > capacity {
                        return Err("metadata buffer too small".into());
                    }
                    // SAFETY: the caller provides capacity writable bytes; lengths checked.
                    unsafe {
                        std::ptr::copy_nonoverlapping(text.as_ptr(), output.cast(), text.len());
                    }
                }
                Ok(i64::try_from(text.len())?)
            }
            7..=9 => Ok(0),
            _ => Err("unsupported provider operation".into()),
        }
    })();
    result.unwrap_or(-1)
}
