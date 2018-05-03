/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate bindgen;
extern crate cc;

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    build_jsapi();
    build_jsglue();
    build_jsapi_bindings();
}

fn find_make() -> OsString {
    if let Some(make) = env::var_os("MAKE") {
        make
    } else {
        match Command::new("gmake").status() {
            Ok(_) => OsStr::new("gmake").to_os_string(),
            Err(_) => OsStr::new("make").to_os_string(),
        }
    }
}

fn build_jsapi() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let mut make = find_make();
    // Put MOZTOOLS_PATH at the beginning of PATH if specified
    if let Some(moztools) = env::var_os("MOZTOOLS_PATH") {
        let path = env::var_os("PATH").unwrap();
        let mut paths = Vec::new();
        paths.extend(env::split_paths(&moztools));
        paths.extend(env::split_paths(&path));
        let new_path = env::join_paths(paths).unwrap();
        env::set_var("PATH", &new_path);
        make = OsStr::new("mozmake").to_os_string();
    }

    let mut cmd = Command::new(make);

    // We're using the MSYS make which doesn't work with the mingw32-make-style
    // MAKEFLAGS, so remove that from the env if present.
    if cfg!(windows) {
        cmd.env_remove("MAKEFLAGS").env_remove("MFLAGS");
    } else if let Some(makeflags) = env::var_os("CARGO_MAKEFLAGS") {
        cmd.env("MAKEFLAGS", makeflags);
    }

    let result = cmd.args(&["-R", "-f", "makefile.cargo"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run `make`");

    assert!(result.success());
    println!("cargo:rustc-link-search=native={}/js/src", out_dir);
    println!("cargo:rustc-link-lib=static=js_static"); // Must come before c++
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=psapi");
        if target.contains("gnu") {
            println!("cargo:rustc-link-lib=stdc++");
        }
    } else if target.contains("apple") || target.contains("freebsd") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
    println!("cargo:outdir={}", out_dir);
}


fn build_jsglue() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        
    cc::Build::new()
        .flag("-std=c++11")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-invalid-offsetof")
        .file("src/jsglue.cpp")
        .include(out.join("dist/include"))
        .compile("jsglue");

    println!("cargo:rerun-if-changed=src/jsglue.cpp");
}

/// Invoke bindgen on the JSAPI headers to produce raw FFI bindings for use from
/// Rust.
///
/// To add or remove which functions, types, and variables get bindings
/// generated, see the `const` configuration variables below.
fn build_jsapi_bindings() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // By default, constructors, destructors and methods declared in .h files are inlined,
    // so their symbols aren't available. Adding the -fkeep-inlined-functions option
    // causes the jsapi library to bloat from 500M to 6G, so that's not an option.
    let mut config = bindgen::CodegenConfig::all();
    config.constructors = false;
    config.destructors = false;
    config.methods = false;
    
    let mut builder = bindgen::builder()
        .rust_target(bindgen::RustTarget::Stable_1_19)
        .header("./src/jsglue.hpp")
        // Translate every enum with the "rustified enum" strategy. We should
        // investigate switching to the "constified module" strategy, which has
        // similar ergonomics but avoids some potential Rust UB footguns.
        .rustified_enum(".*")
        .enable_cxx_namespaces()
        .with_codegen_config(config)
        .rustfmt_bindings(true)
        .clang_arg("-I").clang_arg(out.join("dist/include").to_str().expect("UTF-8"))
        .clang_arg("-x").clang_arg("c++")
        .clang_arg("-std=gnu++14")
        .clang_arg("-fno-sized-deallocation")
        .clang_arg("-DRUST_BINDGEN");

    if cfg!(feature = "debugmozjs") {
        builder = builder
            .clang_arg("-DJS_GC_ZEAL")
            .clang_arg("-DDEBUG")
            .clang_arg("-DJS_DEBUG");
    }

    if cfg!(windows) {
        builder = builder
	    .clang_arg("-fms-compatibility");
    }

    for ty in UNSAFE_IMPL_SYNC_TYPES {
        builder = builder.raw_line(format!("unsafe impl Sync for root::{} {{}}", ty));
    }

    for ty in WHITELIST_TYPES {
        builder = builder.whitelist_type(ty);
    }

    for var in WHITELIST_VARS {
        builder = builder.whitelist_var(var);
    }

    for func in WHITELIST_FUNCTIONS {
        builder = builder.whitelist_function(func);
    }

    for ty in OPAQUE_TYPES {
        builder = builder.opaque_type(ty);
    }

    for ty in BLACKLIST_TYPES {
        builder = builder.blacklist_type(ty);
    }

    for &(module, raw_line) in MODULE_RAW_LINES {
        builder = builder.module_raw_line(module, raw_line);
    }

    let bindings = builder.generate()
        .expect("Should generate JSAPI bindings OK");

    bindings.write_to_file(out.join("jsapi.rs"))
        .expect("Should write bindings to file OK");

    println!("cargo:rerun-if-changed=src/jsglue.hpp");
}

/// JSAPI types for which we should implement `Sync`.
const UNSAFE_IMPL_SYNC_TYPES: &'static [&'static str] = &[
    "JSClass",
    "JSFunctionSpec",
    "JSNativeWrapper",
    "JSPropertySpec",
    "JSTypedMethodJitInfo",
];

/// Types which we want to generate bindings for (and every other type they
/// transitively use).
const WHITELIST_TYPES: &'static [&'static str] = &[
    "DetachDataDisposition",
    "JS::AutoIdVector",
    "JS::AutoObjectVector",
    "JS::CallArgs",
    "JS::CompileOptions",
    "JS::CompartmentOptions",
    "JS::CustomAutoRooter",
    "JS::ForOfIterator",
    "JS::Handle",
    "JS::HandleFunction",
    "JS::HandleId",
    "JS::HandleObject",
    "JS::HandleString",
    "JS::HandleValue",
    "JS::HandleValueArray",
    "JS::IsAcceptableThis",
    "JS::IsArrayAnswer",
    "JS::MutableHandle",
    "JS::MutableHandleObject",
    "JS::MutableHandleValue",
    "JS::NativeImpl",
    "JS::ObjectPrivateVisitor",
    "JS::PromiseState",
    "PropertyCopyBehavior",
    "JS::PropertyDescriptor",
    "JS::RegExpGuard",
    "JS::RootKind",
    "JS::Rooted",
    "JS::RootedObject",
    "JS::SavedFrameResult",
    "JS::SavedFrameSelfHosted",
    "JS::ServoSizes",
    "JS::SymbolCode",
    "JS::TabSizes",
    "JS::TraceKind",
    "JS::TwoByteChars",
    "JS::Value",
    "JS::Zone",
    "JS::shadow::Zone",
    "JSAutoCompartment",
    "JSClass",
    "JSCompartment",
    "JSConstDoubleSpec",
    "JSConstIntegerSpec",
    "JSContext",
    "JSErrorFormatString",
    "JSExnType",
    "JSFlatString",
    "JSFunction",
    "JSFunctionSpec",
    "JSFunctionSpecWithHelp",
    "JSGCParamKey",
    "JSJitGetterCallArgs",
    "JSJitSetterCallArgs",
    "JSNativeWrapper",
    "JSObject",
    "JSONWriteCallback",
    "JSPropertySpec",
    "JSProtoKey",
    "JSScript",
    "JSString",
    "JSStructuredCloneCallbacks",
    "JSType",
    "JSTypedMethodJitInfo",
    "JSValueType",
    "js::ContextFriendFields",
    "js::ESClass",
    "js::PerThreadDataFriendFields",
    "js::shadow::Object",
    "js::shadow::ObjectGroup",
    "js::RegExpGuard",
    "js::ScriptEnvironmentPreparer",
    "js::Type",
    "js::Vector",
    "mozilla::MallocSizeOf",
];

/// Global variables we want to generate bindings to.
const WHITELIST_VARS: &'static [&'static str] = &[
    "JS::NullHandleValue",
    "JS::UndefinedHandleValue",
    "JSCLASS_.*",
    "JSITER_.*",
    "JSID_VOID",
    "JSPROP_.*",
];

/// Functions we want to generate bindings to.
const WHITELIST_FUNCTIONS: &'static [&'static str] = &[
    "glue::JS_NewCompartmentOptions",
    "glue::JS_NewOwningCompileOptions",
    "glue::JS_AsShadowZone",
    "glue::JS_BooleanValue",
    "glue::JS_ValueIsBoolean",
    "glue::JS_ValueToBoolean",
    "glue::JS_DoubleValue",
    "glue::JS_ValueIsDouble",
    "glue::JS_ValueToDouble",
    "glue::JS_Int32Value",
    "glue::JS_ValueIsInt32",
    "glue::JS_ValueToInt32",
    "glue::JS_NullValue",
    "glue::JS_ValueIsNull",
    "glue::JS_UndefinedValue",
    "glue::JS_ValueIsUndefined",
    "JS::AddSizeOfTab",
    "JS::BuildStackString",
    "JS::CallArgsFromVp",
    "JS::CaptureCurrentStack",
    "JS::Evaluate",
    "JS::HeapObjectPostBarrier",
    "JS::HeapValuePostBarrier",
    "JS::InitSelfHostedCode",
    "JS::IsSavedFrame",
    "JS::OrdinaryToPrimitive",
    "JS::RuntimeOptionsRef",
    "JS::SetWarningReporter",
    "JSONWriteCallback",
    "JS_BeginRequest",
    "JS_DefineElement",
    "JS_DefineFunction",
    "JS_DefineFunctions",
    "JS_DefineProperties",
    "JS_DestroyContext",
    "JS_DestroyRuntime",
    "JS_EncodeStringToUTF8",
    "JS_EndRequest",
    "JS_EnterCompartment",
    "JS_EnumerateStandardClasses",
    "JS_GetArrayBufferData",
    "JS_GetArrayBufferViewType",
    "JS_GetContext",
    "JS_GetFloat32ArrayData",
    "JS_GetFloat64ArrayData",
    "JS_GetInt16ArrayData",
    "JS_GetInt32ArrayData",
    "JS_GetInt8ArrayData",
    "JS_GetLatin1StringCharsAndLength",
    "JS_GetTwoByteStringCharsAndLength",
    "JS_GetUint16ArrayData",
    "JS_GetUint32ArrayData",
    "JS_GetUint8ArrayData",
    "JS_GetUint8ClampedArrayData",
    "JS_GlobalObjectTraceHook",
    "JS_Init",
    "JS_InitStandardClasses",
    "JS_LeaveCompartment",
    "JS_LooselyEqual",
    "JS_MayResolveStandardClass",
    "JS_NewArrayBuffer",
    "JS_NewArrayObject",
    "JS_NewCompartmentOptions",
    "JS_NewContext",
    "JS_NewFloat32Array",
    "JS_NewFloat64Array",
    "JS_NewFunction",
    "JS_NewGlobalObject",
    "JS_NewInt16Array",
    "JS_NewInt32Array",
    "JS_NewInt8Array",
    "JS_NewObject",
    "JS_NewOwningCompileOptions",
    "JS_NewRuntime",
    "JS_NewUCStringCopyN",
    "JS_NewUint16Array",
    "JS_NewUint32Array",
    "JS_NewUint8Array",
    "JS_NewUint8ClampedArray",
    "JS_ObjectNotWritten",
    "JS_ReadStructuredClone",
    "JS_ReadTypedArray",
    "JS_RefreshCrossCompartmentWrappers",
    "JS_ReportError",
    "JS_ReportErrorNumber",
    "JS_ResolveStandardClass",
    "JS_SameValue",
    "JS_SetGCParameter",
    "JS_SetNativeStackQuota",
    "JS_ShutDown",
    "JS_StrictlyEqual",
    "JS_StringEqualsAscii",
    "JS_StringHasLatin1Chars",
    "JS_StructuredClone",
    "JS_TransplantObject",
    "JS_TypeOfValue",
    "JS_ValueToFunction",
    "JS_ValueToSource",
    "JS_ValueToConstructor",
    "JS_ValueToObject",
    "JS_WrapObject",
    "JS_WrapValue",
    "JS_WriteString",
    "JS_WriteStructuredClone",
    "JS_WriteTypedArray",
    "js::AssertSameCompartment",
    "js::GetArrayBufferLengthAndData",
    "js::GetArrayBufferViewLengthAndData",
    "js::GetPropertyKeys",
    "js::ToBooleanSlow",
    "js::ToInt32Slow",
    "js::ToInt64Slow",
    "js::ToNumberSlow",
    "js::ToStringSlow",
    "js::ToUint16Slow",
    "js::ToUint32Slow",
    "js::ToUint64Slow",
    "js::ToWindowProxyIfWindow",
    "js::UnwrapArrayBuffer",
    "js::UnwrapArrayBufferView",
    "js::UnwrapFloat32Array",
    "js::UnwrapFloat64Array",
    "js::UnwrapInt16Array",
    "js::UnwrapInt32Array",
    "js::UnwrapInt8Array",
    "js::UnwrapUint16Array",
    "js::UnwrapUint32Array",
    "js::UnwrapUint8Array",
    "js::UnwrapUint8ClampedArray",
    "js::detail::IsWindowSlow",
];

/// Types that should be treated as an opaque blob of bytes whenever they show
/// up within a whitelisted type.
///
/// These are types which are too tricky for bindgen to handle, and/or use C++
/// features that don't have an equivalent in rust, such as partial template
/// specialization.
const OPAQUE_TYPES: &'static [&'static str] = &[
    "JS::ReadOnlyCompileOptions",
    "mozilla::BufferList",
    "mozilla::UniquePtr.*",
    "JS::Rooted<JS::Auto.*Vector.*>",
    "JS::Auto.*Vector",
];

/// Types for which we should NEVER generate bindings, even if it is used within
/// a type or function signature that we are generating bindings for.
const BLACKLIST_TYPES: &'static [&'static str] = &[
    // We provide our own definition because we need to express trait bounds in
    // the definition of the struct to make our Drop implementation correct.
    "JS::Heap",
];

/// Definitions for types that were blacklisted
const MODULE_RAW_LINES: &'static [(&'static str, &'static str)] = &[
    ("root::JS", "pub type Heap<T> = ::jsgc::Heap<T>;")
];
