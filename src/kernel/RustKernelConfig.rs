// src/kernel/RustKernelConfig.rs

//! Módulo de Configuração de Hardware (HAL) para o LightOS.
//! 
//! Este módulo centraliza todas as constantes e configurações de hardware
//! que variam entre as diferentes plataformas suportadas pelo LightOS.
//! 
//! Uso: É importado por drivers e subsistemas de baixo nível (MMU, PIC, etc.).

#![allow(dead_code)]

// ------------------------------------------------------------------------
// --- ⚙️ Configuração da Plataforma (Arquitetura) ---
// ------------------------------------------------------------------------

/// Define a arquitetura do alvo para a qual o kernel está sendo construído.
/// Em um sistema de build real, isso seria definido via variáveis de ambiente/flags do cargo.
pub const TARGET_ARCH: &str = "x86_64"; 

/// Endereço de memória física de inicialização para a camada Rust (após o bootloader C).
/// Geralmente, logo após a área de dados do kernel/módulos.
pub const KERNEL_RUST_START_ADDR: usize = 0x1000000; // 16 MB

// ------------------------------------------------------------------------
// --- 🖥️ Configuração da Tela (VGA/Framebuffer) ---
// ------------------------------------------------------------------------

/// Endereço de MMIO para o buffer de texto VGA (modo texto 80x25).
/// Este endereço é comum na arquitetura x86.
pub const VGA_TEXT_BUFFER_ADDR: usize = 0xb8000;
/// Largura da tela em caracteres (modo texto VGA).
pub const VGA_WIDTH: usize = 80;
/// Altura da tela em caracteres (modo texto VGA).
pub const VGA_HEIGHT: usize = 25;

// ------------------------------------------------------------------------
// --- ⏰ Configuração do Temporizador (Timer) ---
// ------------------------------------------------------------------------

/// Frequência de tique do temporizador de hardware (ex: PIT ou APIC).
/// Define a frequência com que o Kernel recebe interrupções de tempo.
pub const TIMER_FREQUENCY_HZ: u32 = 100; // 100 interrupções por segundo

/// Duração de um tique do temporizador em nanosegundos.
pub const TIMER_TICK_NS: u64 = 1_000_000_000 / TIMER_FREQUENCY_HZ as u64;

// ------------------------------------------------------------------------
// --- 🔊 Configuração de Dispositivos (Exemplo: MMIO Sound) ---
// ------------------------------------------------------------------------

/// Endereço de MMIO (Memory Mapped I/O) base para um dispositivo de som simulado/virtual.
/// (Este seria o valor lido do Barramento PCI/VirtIO na inicialização real)
pub const SOUND_DEVICE_MMIO_BASE: usize = 0xFED0_0000;

// ------------------------------------------------------------------------
// --- ⌨️ Configuração de I/O de Dispositivos Legados ---
// ------------------------------------------------------------------------

/// Porta de I/O (Port I/O) para o Controlador de Interrupção Programável (PIC) Mestre.
pub const PIC_MASTER_COMMAND_PORT: u16 = 0x20;
/// Porta de I/O para o PIC Mestre de Dados (IMR).
pub const PIC_MASTER_DATA_PORT: u16 = 0x21;

/// Porta de I/O para o Teclado/Controlador PS/2.
pub const PS2_DATA_PORT: u16 = 0x60;

// ------------------------------------------------------------------------
// --- 💾 Configuração da Memória (Paging/Heap) ---
// ------------------------------------------------------------------------

/// Tamanho máximo do heap do kernel.
pub const KERNEL_HEAP_SIZE: usize = 2 * 1024 * 1024; // 2 MB

/// Endereço virtual onde o heap do kernel será mapeado.
pub const KERNEL_HEAP_START: usize = 0xC0000000; // Exemplo de endereço alto

// ------------------------------------------------------------------------
// --- 🎯 Configuração da Alocação de Endpoints IPC (Do módulo IPC anterior) ---
// ------------------------------------------------------------------------

/// O próximo ID de Endpoint a ser alocado (deve ser Atomic no código real).
pub const IPC_NEXT_ENDPOINT_ID_START: u64 = 1000;
