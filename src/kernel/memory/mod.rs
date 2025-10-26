// src/kernel/memory/mod.rs

//! Subsistema de Gerenciamento de Memória para o LightOS.
//! 
//! Responsável por inicializar o alocador de heap do kernel e gerenciar
//! os quadros de memória física (frames).

use x86_64::VirtAddr;

// Importa os submódulos
mod frame_alloc;
mod heap_alloc;

// Exporta as APIs públicas
pub use frame_alloc::{
    PhysicalMemoryManager, 
    FrameAllocator, 
    PmmError, 
    PhysicalAddress,
};
pub use heap_alloc::{
    allocator, 
    init_heap, 
    HeapError
};

// ------------------------------------------------------------------------
// --- Funções de Inicialização do Subsistema ---
// ------------------------------------------------------------------------

/// 💾 Inicializa o subsistema de gerenciamento de memória.
/// 
/// Esta função deve ser chamada APÓS a inicialização básica do Paging 
/// pelo código Assembly/C (que garante o mapeamento inicial do Kernel).
///
/// # Safety
/// Esta função é insegura, pois opera em ponteiros de baixo nível e
/// no estado global do kernel. Deve ser chamada apenas uma vez.
pub unsafe fn initialize(
    mut pmm: PhysicalMemoryManager,
    heap_start_addr: VirtAddr,
    heap_size: usize,
) -> Result<(), MemoryError> {
    
    // 1. Inicializa o alocador de quadros físicos (PMM)
    // O PMM deve ser inicializado com as informações da memória (Multiboot2).
    // Aqui, apenas usamos a instância passada.
    pmm.log_initialized_regions();
    
    // 2. Inicializa o Heap do Kernel (K-Heap)
    // O heap é necessário para alocações dinâmicas (ex: Vec, Box).
    match init_heap(heap_start_addr, heap_size) {
        Ok(_) => {
            crate::println!("INFO: Heap do Kernel inicializado com sucesso.");
            Ok(())
        },
        Err(e) => {
            crate::println!("ERRO: Falha ao inicializar o Heap do Kernel: {:?}", e);
            Err(MemoryError::HeapInitFailed)
        }
    }
}

// ------------------------------------------------------------------------
// --- Tipos de Erro Comuns ---
// ------------------------------------------------------------------------

/// Tipo de Erro Comum do Subsistema de Memória.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// Falha na inicialização do Heap.
    HeapInitFailed,
    /// Falha na alocação de um quadro físico.
    FrameAllocationFailed,
    /// Endereço ou mapeamento inválido.
    InvalidMapping,
    /// Erro interno no Paging (ex: página não existe).
    PagingError,
}
