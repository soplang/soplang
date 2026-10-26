//! In-process LLVM object generation from Soplang's textual IR.
//! The FFI surface is deliberately limited to LLVM's C API.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::process::Command;
use std::ptr;
#[cfg(target_os = "linux")]
use std::sync::Mutex;

use crate::error::{external_error, SoplangError};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::error::{runtime_error, tool_start_error};

type Handle = *mut c_void;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn soplang_lld_elf(
        argc: c_int,
        argv: *const *const c_char,
        diagnostics: *mut c_char,
        diagnostics_len: usize,
    ) -> c_int;
}

#[cfg(target_os = "linux")]
pub fn link_elf(
    object: &Path,
    runtime: &Path,
    output: &Path,
    native_paths: &[PathBuf],
) -> Result<(), SoplangError> {
    let runtime_dir = runtime.parent().expect("runtime archive has no parent");
    let bundled_libraries = runtime_dir.join("llvm");
    let candidates = if bundled_libraries.is_dir() {
        vec![runtime_dir.join("crt/Scrt1.o")]
    } else {
        vec![
            runtime_dir.join("crt/Scrt1.o"),
            PathBuf::from("/usr/lib/Scrt1.o"),
            PathBuf::from("/usr/lib/x86_64-linux-gnu/Scrt1.o"),
        ]
    };
    let startup = candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            external_error(
                "Faylka bilowga ee Linux lama helin.",
                "Xirmada Soplang waxaa ka maqan lib/soplang/crt/Scrt1.o.".to_owned(),
            )
        })?;
    let mut args = vec![
        "ld.lld".to_owned(),
        "--hash-style=gnu".to_owned(),
        "--build-id".to_owned(),
        "--eh-frame-hdr".to_owned(),
        "-m".to_owned(),
        "elf_x86_64".to_owned(),
        "-pie".to_owned(),
        "-dynamic-linker".to_owned(),
        "/lib64/ld-linux-x86-64.so.2".to_owned(),
        "-o".to_owned(),
        output.display().to_string(),
        startup.display().to_string(),
    ];
    args.push(object.display().to_string());
    args.push(runtime.display().to_string());
    if bundled_libraries.is_dir() {
        args.push(format!("-L{}", bundled_libraries.display()));
    }
    for path in native_paths {
        args.push(format!("-L{}", path.display()));
    }
    for path in [
        "/lib",
        "/usr/lib",
        "/lib64",
        "/usr/lib64",
        "/lib/x86_64-linux-gnu",
        "/usr/lib/x86_64-linux-gnu",
    ] {
        if Path::new(path).is_dir() {
            args.push(format!("-L{path}"));
        }
    }
    args.extend(
        [
            "-l:libz.so.1",
            "-l:libm.so.6",
            "-l:libc.so.6",
            "-l:libgcc_s.so.1",
        ]
        .map(str::to_owned),
    );
    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.as_str()).expect("link argument contains NUL"))
        .collect();
    let ptrs: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();
    let mut diagnostics = vec![0; 65536];
    // LLD keeps process-global state while linking. Its library entry point is
    // reusable, but overlapping calls corrupt that state.
    static LLD_LOCK: Mutex<()> = Mutex::new(());
    let _guard = LLD_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let code = unsafe {
        soplang_lld_elf(
            ptrs.len() as c_int,
            ptrs.as_ptr(),
            diagnostics.as_mut_ptr(),
            diagnostics.len(),
        )
    };
    if code != 0 {
        let message = unsafe { CStr::from_ptr(diagnostics.as_ptr()) };
        return Err(external_error(
            "LLD ma dhammaystiri karin isku xidhka barnaamijka.",
            message.to_string_lossy().into_owned(),
        ));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn link_macho(
    object: &Path,
    runtime: &Path,
    output: &Path,
    native_paths: &[PathBuf],
) -> Result<(), SoplangError> {
    let dir = runtime.parent().expect("runtime archive has no parent");
    let linker = dir.join("tools/ld64.lld");
    let system = dir.join("sdk/usr/lib/libSystem.tbd");
    if !linker.is_file() || !system.is_file() {
        return Err(runtime_error(
            "Qaybaha isku xidhka macOS ee xirmada Soplang ma dhammaystirna.",
            0,
            0,
        ));
    }
    let mut command = Command::new(&linker);
    command.args([
        "-arch",
        "arm64",
        "-platform_version",
        "macos",
        "15.5",
        "15.5",
    ]);
    command.arg("-syslibroot").arg(dir.join("sdk"));
    command.arg("-o").arg(output).arg(object).arg(runtime);
    command.arg("-L").arg(dir.join("sdk/usr/lib"));
    command
        .arg("-F")
        .arg(dir.join("sdk/System/Library/Frameworks"));
    for path in native_paths {
        command.arg("-L").arg(path);
    }
    command.arg("-lSystem");
    for framework in ["CoreFoundation", "Security", "SystemConfiguration"] {
        command.arg("-framework").arg(framework);
    }
    for stub in ["libz.1.tbd", "libiconv.2.tbd", "libobjc.A.tbd"] {
        command.arg(dir.join("sdk/usr/lib").join(stub));
    }
    let result = command
        .output()
        .map_err(|e| tool_start_error("ld64.lld", &e))?;
    if !result.status.success() {
        return Err(external_error(
            "LLD ma dhammaystiri karin isku xidhka barnaamijka macOS.",
            format!(
                "ld64.lld: {}\n{}{}",
                result.status,
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            ),
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn link_coff(
    object: &Path,
    runtime: &Path,
    output: &Path,
    native_paths: &[PathBuf],
) -> Result<(), SoplangError> {
    let dir = runtime.parent().expect("runtime archive has no parent");
    let zig = dir.join("tools/zig.exe");
    if zig.is_file() {
        let result = Command::new(&zig)
            .args([
                "cc",
                "-target",
                "x86_64-windows-gnu",
                "-Wl,--stack,8388608",
                "-o",
            ])
            .arg(output)
            .arg(object)
            .arg(runtime)
            .args(["-lunwind", "-lgdi32"])
            .output()
            .map_err(|e| tool_start_error("zig", &e))?;
        if !result.status.success() {
            return Err(external_error(
                "LLD ma dhammaystiri karin isku xidhka barnaamijka Windows.",
                format!(
                    "zig cc: {}\n{}{}",
                    result.status,
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                ),
            ));
        }
        return Ok(());
    }
    let linker = dir.join("tools/lld-link.exe");
    let imports = dir.join("native");
    if !linker.is_file() || !imports.is_dir() {
        return Err(runtime_error(
            "Qaybaha isku xidhka Windows ee xirmada Soplang ma dhammaystirna.",
            0,
            0,
        ));
    }
    let mut command = Command::new(&linker);
    command.args(["/NOLOGO", "/SUBSYSTEM:CONSOLE", "/STACK:8388608"]);
    command.arg(format!("/OUT:{}", output.display()));
    command.arg(format!("/LIBPATH:{}", imports.display()));
    for path in native_paths {
        command.arg(format!("/LIBPATH:{}", path.display()));
    }
    command.arg(object).arg(runtime);
    for name in [
        "kernel32.lib",
        "advapi32.lib",
        "gdi32.lib",
        "bcrypt.lib",
        "userenv.lib",
        "ws2_32.lib",
        "ntdll.lib",
        "ole32.lib",
        "shell32.lib",
        "secur32.lib",
        "crypt32.lib",
        "user32.lib",
        "iphlpapi.lib",
        "vcruntime.lib",
        "ucrt.lib",
        "msvcrt.lib",
        "legacy_stdio_definitions.lib",
    ] {
        if imports.join(name).is_file() {
            command.arg(name);
        }
    }
    let result = command
        .output()
        .map_err(|e| tool_start_error("lld-link", &e))?;
    if !result.status.success() {
        return Err(external_error(
            "LLD ma dhammaystiri karin isku xidhka barnaamijka Windows.",
            format!(
                "lld-link: {}\n{}{}",
                result.status,
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            ),
        ));
    }
    Ok(())
}

unsafe extern "C" {
    fn LLVMContextCreate() -> Handle;
    fn LLVMContextDispose(context: Handle);
    fn LLVMCreateMemoryBufferWithMemoryRangeCopy(
        data: *const c_char,
        len: usize,
        name: *const c_char,
    ) -> Handle;
    fn LLVMDisposeMemoryBuffer(buffer: Handle);
    fn LLVMParseIRInContext(
        context: Handle,
        buffer: Handle,
        module: *mut Handle,
        message: *mut *mut c_char,
    ) -> c_int;
    fn LLVMVerifyModule(module: Handle, action: c_int, message: *mut *mut c_char) -> c_int;
    fn LLVMDisposeModule(module: Handle);
    fn LLVMDisposeMessage(message: *mut c_char);
    fn LLVMGetDefaultTargetTriple() -> *mut c_char;
    fn LLVMGetTargetFromTriple(
        triple: *const c_char,
        target: *mut Handle,
        message: *mut *mut c_char,
    ) -> c_int;
    fn LLVMCreateTargetMachine(
        target: Handle,
        triple: *const c_char,
        cpu: *const c_char,
        features: *const c_char,
        opt: c_int,
        reloc: c_int,
        model: c_int,
    ) -> Handle;
    fn LLVMDisposeTargetMachine(machine: Handle);
    fn LLVMCreateTargetDataLayout(machine: Handle) -> Handle;
    fn LLVMDisposeTargetData(data: Handle);
    fn LLVMSetTarget(module: Handle, triple: *const c_char);
    fn LLVMSetModuleDataLayout(module: Handle, data: Handle);
    fn LLVMCreatePassBuilderOptions() -> Handle;
    fn LLVMDisposePassBuilderOptions(options: Handle);
    fn LLVMRunPasses(
        module: Handle,
        passes: *const c_char,
        machine: Handle,
        options: Handle,
    ) -> Handle;
    fn LLVMGetErrorMessage(error: Handle) -> *mut c_char;
    fn LLVMDisposeErrorMessage(message: *mut c_char);
    fn LLVMTargetMachineEmitToMemoryBuffer(
        machine: Handle,
        module: Handle,
        kind: c_int,
        message: *mut *mut c_char,
        buffer: *mut Handle,
    ) -> c_int;
    fn LLVMGetBufferStart(buffer: Handle) -> *const c_char;
    fn LLVMGetBufferSize(buffer: Handle) -> usize;
    #[cfg(target_arch = "x86_64")]
    fn LLVMInitializeX86TargetInfo();
    #[cfg(target_arch = "x86_64")]
    fn LLVMInitializeX86Target();
    #[cfg(target_arch = "x86_64")]
    fn LLVMInitializeX86TargetMC();
    #[cfg(target_arch = "x86_64")]
    fn LLVMInitializeX86AsmPrinter();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64TargetInfo();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64Target();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64TargetMC();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64AsmPrinter();
}

struct Owned(Handle, unsafe extern "C" fn(Handle));

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { (self.1)(self.0) };
        }
    }
}

fn error(context: &str, detail: impl Into<String>) -> SoplangError {
    external_error(
        format!("LLVM ma dhammaystiri karin {context}."),
        detail.into(),
    )
}

unsafe fn message(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return "LLVM ma bixin faahfaahin.".into();
    }
    let result = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { LLVMDisposeMessage(ptr) };
    result
}

unsafe fn initialize_target() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        LLVMInitializeX86TargetInfo();
        LLVMInitializeX86Target();
        LLVMInitializeX86TargetMC();
        LLVMInitializeX86AsmPrinter();
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        LLVMInitializeAArch64TargetInfo();
        LLVMInitializeAArch64Target();
        LLVMInitializeAArch64TargetMC();
        LLVMInitializeAArch64AsmPrinter();
    }
}

/// Returns a native object file. The returned bytes do not depend on LLVM's
/// buffers and can be passed to a linker after this function returns.
pub fn emit_object(ir: &str, opt_level: u8) -> Result<Vec<u8>, SoplangError> {
    if opt_level > 3 {
        return Err(error(
            "hagaajinta",
            "Heerka hagaajintu waa inuu ahaadaa 0 ilaa 3.",
        ));
    }
    let name = c"soplang.ll";
    let cpu = c"generic";
    let features = c"";
    // Every resource below is scoped to this call. LLVM's global target
    // registrations are idempotent, while modules and target machines are not.
    unsafe {
        initialize_target();
        let context = Owned(LLVMContextCreate(), LLVMContextDispose);
        if context.0.is_null() {
            return Err(error("abuuridda macnaha", "LLVM context ma abuurmin."));
        }
        let input =
            LLVMCreateMemoryBufferWithMemoryRangeCopy(ir.as_ptr().cast(), ir.len(), name.as_ptr());
        if input.is_null() {
            return Err(error(
                "abuuridda LLVM IR",
                "LLVM memory buffer ma abuurmin.",
            ));
        }
        let mut module = ptr::null_mut();
        let mut diagnostic = ptr::null_mut();
        // LLVMParseIRInContext consumes the buffer on success and failure.
        // This API is also available in the LLVM 18 release build environment.
        if LLVMParseIRInContext(context.0, input, &mut module, &mut diagnostic) != 0 {
            return Err(error("akhrinta LLVM IR", message(diagnostic)));
        }
        let module = Owned(module, LLVMDisposeModule);
        if LLVMVerifyModule(module.0, 2, &mut diagnostic) != 0 {
            return Err(error("hubinta LLVM IR", message(diagnostic)));
        }
        if !diagnostic.is_null() {
            LLVMDisposeMessage(diagnostic);
        }
        let triple_ptr = LLVMGetDefaultTargetTriple();
        if triple_ptr.is_null() {
            return Err(error(
                "doorashada bartilmaameedka",
                "LLVM target triple ma jiro.",
            ));
        }
        let triple = CStr::from_ptr(triple_ptr).to_owned();
        LLVMDisposeMessage(triple_ptr);
        let mut target = ptr::null_mut();
        diagnostic = ptr::null_mut();
        if LLVMGetTargetFromTriple(triple.as_ptr(), &mut target, &mut diagnostic) != 0 {
            return Err(error("doorashada bartilmaameedka", message(diagnostic)));
        }
        let machine = Owned(
            LLVMCreateTargetMachine(
                target,
                triple.as_ptr(),
                cpu.as_ptr(),
                features.as_ptr(),
                opt_level.into(),
                2, // LLVMRelocPIC: match clang's default position-independent object.
                0,
            ),
            LLVMDisposeTargetMachine,
        );
        if machine.0.is_null() {
            return Err(error(
                "abuuridda bartilmaameedka",
                "LLVM target machine ma abuurmin.",
            ));
        }
        LLVMSetTarget(module.0, triple.as_ptr());
        let layout = Owned(LLVMCreateTargetDataLayout(machine.0), LLVMDisposeTargetData);
        LLVMSetModuleDataLayout(module.0, layout.0);
        let options = Owned(
            LLVMCreatePassBuilderOptions(),
            LLVMDisposePassBuilderOptions,
        );
        let pipeline =
            CString::new(format!("default<O{opt_level}>")).expect("static pass pipeline");
        let pass_error = LLVMRunPasses(module.0, pipeline.as_ptr(), machine.0, options.0);
        if !pass_error.is_null() {
            let text = LLVMGetErrorMessage(pass_error);
            let description = CStr::from_ptr(text).to_string_lossy().into_owned();
            LLVMDisposeErrorMessage(text);
            return Err(error("hagaajinta LLVM IR", description));
        }
        let mut output = ptr::null_mut();
        diagnostic = ptr::null_mut();
        if LLVMTargetMachineEmitToMemoryBuffer(machine.0, module.0, 1, &mut diagnostic, &mut output)
            != 0
        {
            return Err(error("samaynta faylka mashiinka", message(diagnostic)));
        }
        let output = Owned(output, LLVMDisposeMemoryBuffer);
        let bytes = std::slice::from_raw_parts(
            LLVMGetBufferStart(output.0).cast::<u8>(),
            LLVMGetBufferSize(output.0),
        );
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::emit_object;

    #[test]
    fn emits_an_elf_object_without_clang() {
        let object = emit_object("define i32 @main() { ret i32 0 }", 2).unwrap();
        #[cfg(target_os = "linux")]
        assert!(object.starts_with(b"\x7fELF"));
        #[cfg(target_os = "macos")]
        assert!(!object.is_empty());
        #[cfg(target_os = "windows")]
        assert!(!object.is_empty());
    }
}
