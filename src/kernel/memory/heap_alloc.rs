// src/kernel/memory/heap_alloc.rs

use x86_64::VirtAddr;
use linked_list_allocator::LockedHeap; // Crate popular de alocador no_std

/// Tipo de Erro para o Alocador de Heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapError {
    AlreadyInitialized,
    InsufficientSize,
}

// ------------------------------------------------------------------------
// --- Alocador Global ---
// ------------------------------------------------------------------------

/// 🧠 O Alocador Global de Heap do Kernel.
/// 
/// Usa 'LockedHeap' para garantir o acesso seguro e thread-safe, necessário 
/// em ambientes multitarefa (threads).
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// ------------------------------------------------------------------------
// --- Inicialização do Heap ---
// ------------------------------------------------------------------------

/// 📈 Inicializa o alocador de Heap do Kernel.
/// 
/// O Heap do Kernel é uma área de memória virtual já mapeada pelo MMU 
/// no código C/Assembly.
///
/// # Safety
/// Inseguro, pois modifica o estado global do alocador e opera em ponteiros brutos.
pub unsafe fn init_heap(heap_start_addr: VirtAddr, heap_size: usize) -> Result<(), HeapError> {
    if heap_size < 1024 {
        return Err(HeapError::InsufficientSize);
    }
    
    // Converte o endereço virtual para um ponteiro mutável
    let heap_start_ptr = heap_start_addr.as_mut_ptr();
    
    // Inicializa o alocador global
    ALLOCATOR.lock().init(heap_start_ptr, heap_size);
    
    Ok(())
}
