use crate::args_parser::LaunchTarget::Jar;
use crate::args_parser::LauncherArg;
use crate::jvm::jvmti_init::{load_ext_runtime, set_callbacks};
use crate::jvm::launcher_helper::{find_launcher_helper_from_env, JvmLauncherHelper};
use crate::util::jvm_util::JvmWrapper;
use jni::objects::{JClass, JObject};
use jni::sys::jsize;
use jni::JNIEnv;
use jni::JNIVersion;
use crate::jni_result_expect;

const JAVA_CLASS_PATH_VM_ARG_PREFIX: &str = "-Djava.class.path=";

pub fn jvm_launch(launcher_arg: &LauncherArg) {
    let vm_ops = launcher_arg.vm_args();

    let mut args_builder = jni::InitArgsBuilder::new()
        .version(JNIVersion::V8);
    let target = launcher_arg.target();
    let mut java_class_path = String::from(JAVA_CLASS_PATH_VM_ARG_PREFIX);
    if let Jar(jar) = &target {
        java_class_path.push_str(jar.path());
        // args_builder = args_builder.option(&java_class_path);
        (jar.main_class().replace('.', "/"), jar.signature())
    } else {
        // todo not currently supported
        panic!("not currently supported run class")
    };
    if launcher_arg.server() {
        args_builder = args_builder.option("-server");
    }
    for item in vm_ops.iter() {
        args_builder = args_builder.option(item.trim());
    };
    let init_args = args_builder
        .option(&java_class_path)
        .build()
        .expect("init Jvm args failed");

    // let jvm = JavaVM::new(init_args).unwrap();
    // let jvm = JavaVM::with_libjvm(init_args,
    //                               || StartJvmResult::Ok(PathBuf::from("D:\\software\\install\\Java\\jdk1.8.0_202\\jre\\bin\\server\\jvm.dll"))).unwrap();

    let wrapper = JvmWrapper::load_jvm().unwrap();
    // let wrapper = JvmWrapper::load_jvm_with("D:\\software\\install\\Java\\jdk1.8.0_202\\jre\\bin\\server\\jvm.dll").unwrap();


    let (jvm, mut env) = wrapper.create_java_vm(init_args).unwrap();
    let vers = env.get_version().expect("get jvm version failed!");
    set_callbacks(&jvm, vers.into());

    // get JNI env
    // let mut env = jvm.attach_current_thread().expect("get jni env failed");
    // let mut env = jvm.get_env().unwrap();

    let env_ref = &mut env;
    load_ext_runtime(&jvm, env_ref);

    let app_args = launcher_arg.app_args();

    let helper = find_launcher_helper_from_env(env_ref);

    let main_class = jni_result_expect!(env_ref, helper.check_and_load_main(env_ref, target), &format!("can not load main class: {}", target.main_class()));
    // let main_class = env.find_class(&main_class_name).expect(&format!("not found main class: {}", &main_class_name));

    call_main(env_ref, &main_class, app_args);

    unsafe {
        jvm.detach_current_thread();
        jni_result_expect!(env_ref, jvm.destroy());
    }
}

fn call_main(env: &mut JNIEnv, main_class: &JClass, app_args: &Vec<String>) {
    let args = jni_result_expect!(env, env.new_object_array(jsize::from(app_args.len() as i32), "java/lang/String", JObject::null()));

    for (i, item) in app_args.iter().enumerate() {
        jni_result_expect!(env, env.set_object_array_element(&args, jsize::from(i as i32), jni_result_expect!(env, env.new_string(item))));
    }

    let params = [jni::objects::JValue::Object(&args)];
    jni_result_expect!(env, env.call_static_method(main_class, "main", "([Ljava/lang/String;)V", &params));
    // match env.call_static_method(main_class, "main", "([Ljava/lang/String;)V",
    //                        &params) {
    //     Ok(_) => {
    //     }
    //     Err(err) => {
    //         if let Ok(true) = env.exception_check() {
    //             env.exception_describe().expect("print error failed!");
    //             env.exception_clear().unwrap();
    //         } else {
    //             eprintln!("main method execution failed: {err}")
    //         }
    //     }
    // }
}