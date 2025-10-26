// src/kernel/syscall/mod.rs

//! Subsistema de Chamadas de Sistema (Syscalls) para o LightOS.
//! 
//! Define o vetor de Syscalls e o dispatcher para as chamadas de nível de usuário.
//! No x86_64, Syscalls são geralmente acionadas por uma instrução como `SYSCALL` 
//! ou por uma interrupção de software (ex: INT 0x80).

use x86_64::structures::idt::InterruptStackFrame;

// ------------------------------------------------------------------------
// --- Definições de Syscall ---
// ------------------------------------------------------------------------

/// 🔢 Enumeração de IDs de Chamada de Sistema.
/// * Os IDs são usados pelo Userspace para especificar qual serviço do Kernel solicitar.
#[repr(u64)]
pub enum SyscallId {
    /// Saída para o console.
    PrintString = 1,
    /// Termina a tarefa atual.
    Exit = 2,
    /// Cria uma nova tarefa.
    SpawnTask = 3,
    /// Faz uma chamada para o Trusted Execution Environment (TEE).
    TrustyCall = 100,
    /// ID Inválido.
    Invalid = 999,
}

/// 📝 Estrutura que contém os argumentos de uma Syscall.
/// * Em x86_64, os argumentos são passados em registradores (RDI, RSI, RDX, R10, R8, R9).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SyscallArgs {
    pub arg1: u64, // RDI
    pub arg2: u64, // RSI
    pub arg3: u64, // RDX
    pub arg4: u64, // R10
    pub arg5: u64, // R8
    pub arg6: u64, // R9
}

// ------------------------------------------------------------------------
// --- Funções de Dispatcher e Handler ---
// ------------------------------------------------------------------------

/// ⚙️ Inicializa o subsistema de Syscalls (Se necessário, configura o registro SYSCALL/SYSRET).
pub fn initialize() {
    // Para INT 0x80 (software interrupt), a IDT já está configurada no interrupts::mod.rs.
    // Para SYSCALL/SYSRET (abordagem moderna), seriam necessários registradores MSR.
    crate::println!("INFO: Subsistema de Syscalls pronto.");
}

/// 📞 O dispatcher principal de chamadas de sistema.
/// 
/// Esta função é chamada pelo Assembly/FFI quando uma Syscall é acionada.
///
/// # Argumentos
/// * `id`: O ID da Syscall (geralmente passado em RAX).
/// * `args`: Os argumentos passados pelos registradores.
///
/// # Retorno
/// O valor de retorno da Syscall (geralmente retornado em RAX).
pub fn syscall_dispatcher(id: u64, args: SyscallArgs) -> u64 {
    
    let syscall_id = match id {
        1 => SyscallId::PrintString,
        2 => SyscallId::Exit,
        3 => SyscallId::SpawnTask,
        100 => SyscallId::TrustyCall,
        _ => SyscallId::Invalid,
    };

    // crate::println!("SYSCALL: ID {:?} (Args: {:?})", syscall_id, args);

    match syscall_id {
        SyscallId::PrintString => {
            // Syscall 1: PrintString(ptr: *const u8, len: usize)
            // Assumimos que o Kernel pode acessar o ponteiro do Userspace (necessita de MMU/Paging)
            // Se o kernel e userspace usam o mesmo espaço de endereçamento virtual (monolítico)
            let ptr = args.arg1 as *const u8;
            let len = args.arg2 as usize;
            
            unsafe {
                if let Ok(s) = core::str::from_utf8(core::slice::from_raw_parts(ptr, len)) {
                    crate::println!("[APP LOG] {}", s);
                    return 0;
                }
            }
            1 // Retorna 1 (Erro)
        }
        
        SyscallId::Exit => {
            // Syscall 2: Exit(status: u64)
            crate::println!("[APP] Tarefa solicitou Exit com status: {}", args.arg1);
            // Chama o Scheduler para terminar a tarefa atual e agendar outra
            // task::exit_current_task(args.arg1);
            0
        }
        
        SyscallId::SpawnTask => {
            // Syscall 3: SpawnTask(entry_point_addr: u64)
            // Cria uma nova tarefa com o endereço de entrada fornecido
            // task::spawn_task(unsafe { core::mem::transmute(args.arg1) });
            0
        }

        SyscallId::TrustyCall => {
            // Syscall 100: TrustyCall(handle: u64, command_ptr: *const u8, ...)
            // Encaminha a chamada para o módulo TEE/Trusty
            crate::println!("[TRUSTY] Chamada para TEE (Handle: {})", args.arg1);
            // ffi::dispatch_trusty_call(...)
            0
        }

        SyscallId::Invalid => {
            crate::println!("[ERRO] Syscall ID inválido: {}", id);
            // Retorna um código de erro Syscall
            0xFFFF_FFFF_FFFF_FFFF 
        }
    }
}
