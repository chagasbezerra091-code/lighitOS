// src/kernel/memory/frame_alloc.rs

use x86_64::{
    structures::paging::{PageSize, Size4KiB, FrameAllocator},
    PhysAddr,
};
use core::fmt;

// Definição de tipos para clareza
pub type PhysicalAddress = PhysAddr;

/// Tipo de Erro para o Gerenciador de Quadros Físicos (PMM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmmError {
    NoFreeFrames,
    RegionNotFound,
    InvalidInfo,
}

/// 🧠 Implementação simples de um Gerenciador de Quadros Físicos (PMM).
/// * Em um kernel real, isso seria um Bitmap Allocator ou Buddy Allocator.
pub struct PhysicalMemoryManager {
    // Lista de regiões de memória disponíveis (obtidas do Multiboot2 ou UEFI)
    // Usamos um array estático para simplicidade em no_std.
    available_regions: [(PhysAddr, u64); 32], 
    next_free_frame: PhysAddr,
    region_count: usize,
}

impl PhysicalMemoryManager {
    /// 🏭 Cria uma nova instância do PMM (vazia, para ser preenchida).
    pub const fn new() -> Self {
        const EMPTY_REGION: (PhysAddr, u64) = (PhysAddr::new_truncate(0), 0);
        PhysicalMemoryManager {
            available_regions: [EMPTY_REGION; 32],
            next_free_frame: PhysAddr::new_truncate(0),
            region_count: 0,
        }
    }

    /// ➕ Adiciona uma região de memória livre à lista.
    /// * Chamado durante a inicialização, usando as informações do Multiboot2.
    ///
    /// # Safety
    /// Inseguro, pois modifica o estado de memória de baixo nível.
    pub unsafe fn add_available_region(&mut self, start: PhysAddr, len: u64) {
        if self.region_count < self.available_regions.len() {
            self.available_regions[self.region_count] = (start, len);
            self.region_count += 1;
            
            // Inicializa o 'next_free_frame' se ainda não foi feito.
            if self.next_free_frame.is_zero() {
                self.next_free_frame = start;
            }
        } else {
            crate::println!("WARN: PMM atingiu o limite de regiões.");
        }
    }
    
    /// 📋 Loga as regiões de memória inicializadas.
    pub fn log_initialized_regions(&self) {
        crate::println!("--- PMM: Regiões de Memória Disponíveis ---");
        for i in 0..self.region_count {
            let (start, len) = self.available_regions[i];
            crate::println!("Região {}: Start={:#x}, Len={} MB", 
                i, start.as_u64(), len / 1024 / 1024);
        }
        crate::println!("------------------------------------------");
    }
}

// Implementa o Trait FrameAllocator do x86_64
unsafe impl FrameAllocator<Size4KiB> for PhysicalMemoryManager {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        // Implementação simplificada: aloca frames sequencialmente.
        let frame = PhysFrame::containing_address(self.next_free_frame);
        
        // Verificação simplificada se o frame está dentro de uma região disponível
        // (Lógica real exigiria verificar todas as regiões e marcar frames como usados)
        let frame_addr = frame.start_address().as_u64();
        let frame_size = Size4KiB::SIZE;

        for i in 0..self.region_count {
            let (start, len) = self.available_regions[i];
            let end = start.as_u64() + len;

            if frame_addr >= start.as_u64() && (frame_addr + frame_size) <= end {
                // Encontrado: Atualiza para o próximo frame
                self.next_free_frame = PhysAddr::new(frame_addr + frame_size);
                return Some(frame);
            }
        }
        
        // Nenhum frame livre na região sequencial atual.
        None
    }
}
