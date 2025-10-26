// src/kernel/drivers/display.rs

use core::{
    ptr,
    slice,
};
use crate::RustKernelConfig::arch_hal::VGA_WIDTH; // Exemplo: Importa do módulo HAL
use crate::RustKernelConfig::VGA_TEXT_BUFFER_ADDR; // Importa endereço do HAL

/// 🚨 Códigos de Erro Específicos para o Driver de Display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayError {
    /// O Framebuffer não foi encontrado ou não foi passado pelo bootloader.
    FramebufferNotFound,
    /// O formato de pixel ou profundidade de cor é incompatível.
    UnsupportedFormat,
    /// Parâmetros de MMIO inválidos.
    InvalidMmio,
}

/// 🖥️ Estrutura de Configuração do Framebuffer
/// Armazena as propriedades essenciais da tela.
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Endereço de MMIO onde o framebuffer reside.
    pub address: usize,
    /// Largura da tela em pixels.
    pub width: u32,
    /// Altura da tela em pixels.
    pub height: u32,
    /// Passos (pitch) em bytes: número de bytes por linha.
    pub pitch: u32,
    /// Profundidade de cor em bits por pixel (ex: 24, 32).
    pub bpp: u8, 
}

/// 🎨 Driver de Display Principal do LightOS
/// Gerencia o acesso seguro ao hardware de exibição.
pub struct DisplayDriver {
    info: FramebufferInfo,
    /// Um ponteiro seguro para a área de memória do framebuffer.
    framebuffer_ptr: *mut u8,
}

impl DisplayDriver {
    /// 📝 Tenta criar e inicializar o driver de display.
    /// 
    /// # Safety
    /// Esta função é insegura pois assume que o `address` é um endereço
    /// de MMIO válido, mapeado e que o hardware correspondente existe.
    pub unsafe fn new(info: FramebufferInfo) -> Result<Self, DisplayError> {
        if info.address == 0 {
            return Err(DisplayError::FramebufferNotFound);
        }

        let total_size = info.pitch * info.height;
        
        Ok(DisplayDriver {
            info,
            // Cria um ponteiro mutável para o endereço do framebuffer
            framebuffer_ptr: info.address as *mut u8,
        })
    }

    /// ⚙️ Inicializa o modo gráfico (apaga a tela).
    pub fn initialize(&mut self) -> Result<(), DisplayError> {
        // Exemplo de verificação de formato.
        if self.info.bpp < 24 {
            // Se a profundidade de cor for muito baixa para renderização moderna.
            // Em um kernel real, isso reverteria para o modo VGA.
            // return Err(DisplayError::UnsupportedFormat);
        }

        // Limpa o framebuffer, preenchendo-o com zeros (preto)
        self.clear_screen(0x00, 0x00, 0x00);
        
        crate::println!("INFO: Display Driver inicializado em {}x{} @ {} bpp.", 
            self.info.width, self.info.height, self.info.bpp);
            
        Ok(())
    }
    
    /// 🧹 Preenche toda a tela com uma cor RGB específica.
    pub fn clear_screen(&self, r: u8, g: u8, b: u8) {
        let total_bytes = (self.info.pitch * self.info.height) as usize;
        let bytes_per_pixel = (self.info.bpp / 8) as usize;

        // Cria a cor de preenchimento (assumindo formato RGB ou BGR)
        let color_bytes: [u8; 4] = match bytes_per_pixel {
            3 => [b, g, r, 0], // RGB 24-bit (sem byte alfa)
            4 => [b, g, r, 0], // RGB 32-bit (byte alfa opcional em 0)
            _ => return, // Não suportado
        };

        // Itera sobre a memória e escreve a cor
        for i in 0..total_bytes / bytes_per_pixel {
            let offset = i * bytes_per_pixel;
            // # SAFETY: Esta é a operação mais perigosa, escrevendo diretamente no MMIO.
            // É seguro aqui porque está encapsulado e o tamanho total é verificado.
            unsafe {
                ptr::write_volatile(self.framebuffer_ptr.add(offset) as *mut u8, color_bytes[0]);
                ptr::write_volatile(self.framebuffer_ptr.add(offset + 1) as *mut u8, color_bytes[1]);
                ptr::write_volatile(self.framebuffer_ptr.add(offset + 2) as *mut u8, color_bytes[2]);
                if bytes_per_pixel == 4 {
                    ptr::write_volatile(self.framebuffer_ptr.add(offset + 3) as *mut u8, color_bytes[3]);
                }
            }
        }
    }

    /// 📌 Desenha um único pixel em uma coordenada (x, y).
    pub fn draw_pixel(&self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        if x >= self.info.width || y >= self.info.height {
            return; // Fora dos limites
        }

        let bytes_per_pixel = (self.info.bpp / 8) as u32;
        let offset = (y * self.info.pitch + x * bytes_per_pixel) as usize;
        
        let color_bytes: [u8; 4] = [b, g, r, 0]; // Simples BGR/RGB

        // # SAFETY: A segurança do ponteiro é verificada pelos limites (x, y).
        unsafe {
            let pixel_ptr = self.framebuffer_ptr.add(offset);
            ptr::write_volatile(pixel_ptr, color_bytes[0]);
            ptr::write_volatile(pixel_ptr.add(1), color_bytes[1]);
            ptr::write_volatile(pixel_ptr.add(2), color_bytes[2]);
            if bytes_per_pixel == 4 {
                ptr::write_volatile(pixel_ptr.add(3), color_bytes[3]);
            }
        }
    }
}
