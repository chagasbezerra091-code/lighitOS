// src/kernel/memory/vma.rs

/*
 * Copyright 2024 Chagas Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Gerenciamento de Áreas de Memória Virtual (VMA) para tarefas de Userspace.
//! 
//! Uma VMA representa um segmento contíguo de memória virtual dentro do 
//! espaço de endereçamento de uma tarefa, como o código, dados, pilha ou heap.

use x86_64::{VirtAddr, structures::paging::PageTableFlags};
use alloc::collections::BTreeMap;

/// 📑 Uma entrada de Área de Memória Virtual (VMA).
/// 
/// Define a permissão e o tipo de memória de um segmento de endereço virtual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualMemoryArea {
    /// Endereço virtual inicial da área.
    pub start_addr: VirtAddr,
    /// Tamanho da área em bytes.
    pub size: usize,
    /// Flags da tabela de páginas (ex: WRITE, USER_ACCESSIBLE, NO_EXECUTE).
    pub flags: PageTableFlags,
    /// Tipo da área (ex: Código, Dados, Pilha).
    pub area_type: VMA_Type,
}

/// 🏷️ Tipos de Áreas de Memória Virtual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMA_Type {
    /// Área de código (somente leitura e execução).
    Code,
    /// Área de dados (leitura e escrita).
    Data,
    /// Pilha (leitura e escrita).
    Stack,
    /// Heap (leitura e escrita).
    Heap,
    /// Memória compartilhada ou mapeada.
    MappedFile,
}

/// 🌳 Gerenciador de Áreas de Memória Virtual de uma Tarefa.
/// 
/// Este mapa armazena todas as VMAs que compõem o espaço de endereçamento virtual
/// do Userspace para uma tarefa específica.
pub struct VMA_Manager {
    /// O mapa armazena as VMAs, indexadas pelo seu endereço virtual inicial.
    areas: BTreeMap<VirtAddr, VirtualMemoryArea>,
}

impl VMA_Manager {
    /// Cria um novo gerenciador de VMA vazio.
    pub fn new() -> Self {
        VMA_Manager {
            areas: BTreeMap::new(),
        }
    }

    /// ➕ Adiciona uma nova área de memória virtual.
    /// 
    /// Retorna `Ok(())` se a área foi adicionada, ou `Err(VMA_Error)` se houver sobreposição.
    pub fn add_area(&mut self, area: VirtualMemoryArea) -> Result<(), VMA_Error> {
        // Verifica sobreposição com áreas existentes (simplificado: verifica apenas o endereço inicial)
        if self.areas.contains_key(&area.start_addr) {
            return Err(VMA_Error::AreaAlreadyExists);
        }

        // TODO: Lógica de sobreposição mais complexa é necessária (verificar se o
        // intervalo [start_addr, start_addr + size) intersecta qualquer VMA existente).
        
        self.areas.insert(area.start_addr, area);
        Ok(())
    }

    /// 🔍 Procura uma VMA que contenha o endereço virtual fornecido.
    pub fn find_area(&self, addr: VirtAddr) -> Option<&VirtualMemoryArea> {
        // Encontra o VMA cujo endereço inicial é menor ou igual a `addr`.
        if let Some((&start_addr, area)) = self.areas.range(..=addr).last() {
            let end_addr = start_addr + area.size;
            
            // Verifica se o endereço está DENTRO do intervalo do VMA
            if addr >= start_addr && addr < end_addr {
                return Some(area);
            }
        }
        None
    }
    
    /// 🗺️ Mapeia a VMA para frames físicos, se necessário.
    /// 
    /// Esta é a função principal chamada pelo Page Fault Handler ao lidar com 
    /// alocação sob demanda (demand paging) ou COW (Copy-on-Write).
    pub fn map_vma_page(&self, fault_addr: VirtAddr) -> Result<(), VMA_Error> {
        use crate::memory::paging::map_page;
        use crate::memory::frame_alloc::FRAME_ALLOCATOR;
        use x86_64::structures::paging::Size4KiB;

        let page = x86_64::structures::paging::Page::<Size4KiB>::containing_address(fault_addr);

        // 1. Encontrar o VMA correspondente
        let area = self.find_area(fault_addr)
            .ok_or(VMA_Error::NoAreaFound)?;
        
        // 2. Alocar um frame físico (demanda)
        let frame = unsafe { 
            FRAME_ALLOCATOR.lock().allocate_frame()
                .ok_or(VMA_Error::OOM) 
        }?;

        // 3. Mapear a página virtual para o frame físico com as permissões do VMA
        unsafe {
            map_page(page, frame, area.flags);
        }

        Ok(())
    }
}

/// ❌ Erros de VMA.
#[derive(Debug)]
pub enum VMA_Error {
    AreaAlreadyExists,
    NoAreaFound,
    OOM, // Out of Memory (Falha ao alocar frame físico)
    // Overlap, // Para a lógica de sobreposição futura
}
