// src/kernel/memory/paging.rs

//! Módulo de Gerenciamento de Paging (MMU) para x86_64.
//! 
//! Responsável por manipular as Tabelas de Páginas e configurar os mapeamentos
//! de memória virtual para física.

use x86_64::{
    structures::paging::{
        Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Mapper,
        OffsetPageTable, FrameAllocator, page_table,
    },
    PhysAddr, VirtAddr,
};
use core::ptr::NonNull;

use super::{frame_alloc::PhysicalMemoryManager, MemoryError};

// ------------------------------------------------------------------------
// --- Endereços de Configuração ---
// ------------------------------------------------------------------------

/// Endereço de mapeamento do Kernel (Higher Half Base)
/// (Deve ser o mesmo que KERNEL_HH_BASE em x86_64_arch.hal)
const KERNEL_OFFSET: u64 = 0xFFFF_8000_0000_0000;

// ------------------------------------------------------------------------
// --- Gerenciador de Mapeamento Principal ---
// ------------------------------------------------------------------------

/// 🗺️ Alias para o nosso gerenciador de mapeamento de páginas.
pub type KernelMapper = OffsetPageTable<'static>;

/// 🏭 Cria um novo gerenciador de mapeamento de páginas (KernelMapper).
/// 
/// Assume que o mapeamento recursivo de 4 níveis já está configurado e acessível.
///
/// # Safety
/// Esta função é insegura porque o chamador deve garantir que:
/// 1. As tabelas de páginas de 4 níveis estão ativas (via registro CR3).
/// 2. O `physical_memory_offset` é o endereço base onde a memória física 
///    está mapeada para o espaço virtual do kernel.
pub unsafe fn init_kernel_mapper() -> KernelMapper {
    let phys_offset = VirtAddr::new(KERNEL_OFFSET);
    let p4_table = active_level_4_table(phys_offset);
    
    // Cria e retorna o OffsetPageTable.
    OffsetPageTable::new(p4_table, phys_offset)
}


/// 🔗 Retorna uma referência mutável à tabela de páginas de Nível 4 ativa.
///
/// # Safety
/// Deve ser chamado apenas após a inicialização do Paging e o mapeamento
/// do Kernel (`KERNEL_OFFSET`) ser válido.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // 1. Ler o registro CR3 (que contém o endereço físico da P4).
    let (p4_table_frame, _) = Cr3::read();

    let phys = p4_table_frame.start_address();
    
    // 2. Calcular o endereço virtual da P4 usando o offset do kernel.
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // 3. Retorna uma referência mutável estática.
    &mut *page_table_ptr
}

// ------------------------------------------------------------------------
// --- Funções de Mapeamento de Alto Nível ---
// ------------------------------------------------------------------------

/// 🗺️ Mapeia um endereço virtual para um endereço físico, alocando um novo frame,
/// usando o gerenciador de mapeamento e o alocador de frames.
pub fn create_mapping(
    page: Page<Size4KiB>,
    mapper: &mut KernelMapper,
    allocator: &mut PhysicalMemoryManager,
    flags: PageTableFlags,
) -> Result<(), MemoryError> {
    
    // 1. Aloca um frame físico livre
    let frame: PhysFrame<Size4KiB> = match allocator.allocate_frame() {
        Some(f) => f,
        None => return Err(MemoryError::FrameAllocationFailed),
    };
    
    // 2. Mapeia a página virtual para o frame físico
    let map_result = unsafe {
        // SAFETY: Assumimos que o frame alocado é válido. O alocador garante que ele é livre.
        mapper.map_to(page, frame, flags, allocator)
    };

    // 3. Garante que o mapeamento foi bem-sucedido e aplica o TLB flush (necessário)
    match map_result {
        Ok(tlb_flush) => {
            tlb_flush.flush();
            Ok(())
        },
        Err(_) => Err(MemoryError::PagingError),
    }
}


/// 💻 Inicializa o subsistema de Paging e o Heap.
/// * Esta é a função que será chamada em `kernel_main`.
///
/// # Safety
/// É altamente inseguro, pois modifica tabelas de páginas globais e inicializa o heap.
pub unsafe fn init_paging_and_heap(
    multiboot2_info_ptr: u64,
    mut pmm: PhysicalMemoryManager,
    heap_start_addr: VirtAddr,
    heap_size: usize,
) -> Result<(), MemoryError> {
    
    // 1. Inicializa o PMM (Gerenciador de Quadros Físicos)
    // * Aqui você leria o multiboot2_info_ptr para preencher as regiões livres do PMM.
    // Exemplo: pmm.add_available_region(...); 
    
    // 2. Inicializa o Kernel Mapper
    let mut mapper = init_kernel_mapper();
    
    crate::println!("INFO: Kernel Mapper inicializado. (Offset: {:#x})", KERNEL_OFFSET);

    // 3. Configuração de Mapeamento Inicial (Exemplo: Mapear o Framebuffer)
    // (O Framebuffer é MMIO e precisa ser mapeado no espaço virtual do Kernel)
    let fb_phys_addr = PhysAddr::new(0xE000_0000); // Exemplo de endereço de MMIO do FB
    let fb_virt_addr = VirtAddr::new(KERNEL_OFFSET + 0x100_0000); // Endereço virtual
    
    let fb_page: Page = Page::containing_address(fb_virt_addr);
    let fb_frame: PhysFrame = PhysFrame::containing_address(fb_phys_addr);

    // Mapear o frame do FB para a página virtual, tornando-o acessível.
    let fb_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

    let map_result = mapper.map_to(fb_page, fb_frame, fb_flags, &mut pmm);

    match map_result {
        Ok(tlb_flush) => {
            tlb_flush.flush();
            crate::println!("INFO: Framebuffer mapeado para {:#x}", fb_virt_addr.as_u64());
        },
        Err(e) => {
            crate::println!("ERRO: Falha ao mapear Framebuffer: {:?}", e);
            return Err(MemoryError::PagingError);
        }
    }

    // 4. Inicializa o Heap do Kernel (K-Heap)
    // O Heap deve ser inicializado APÓS ter sido mapeado na tabela de páginas.
    match super::init_heap(heap_start_addr, heap_size) {
        Ok(_) => {
            crate::println!("INFO: Heap do Kernel inicializado com sucesso.");
            Ok(())
        },
        Err(_) => Err(MemoryError::HeapInitFailed)
    }
}

// ------------------------------------------------------------------------
// --- Teste de Memória (Exemplo) ---
// ------------------------------------------------------------------------

/// 🧪 Função de teste simples para alocação de memória (após inicialização do Heap).
pub fn run_memory_tests() {
    use alloc::{boxed::Box, vec, vec::Vec};
    
    let heap_value = Box::new(42);
    crate::println!("TEST: Heap (Box) alocado com valor: {}", *heap_value);
    
    let mut vec_heap = vec![1, 2, 3];
    vec_heap.push(4);
    crate::println!("TEST: Heap (Vec) alocado com tamanho: {}", vec_heap.len());
}
