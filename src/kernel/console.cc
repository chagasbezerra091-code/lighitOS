// src/kernel/console.cc
#include "console.h"
#include <stdarg.h> // Para funções de formato (se implementadas)
#include <string.h> // Para memmove/memset

// Constantes de Configuração (Melhor lidas de RustKernelConfig.hal, mas definidas aqui para C++)
const size_t VGA_WIDTH = 80;
const size_t VGA_HEIGHT = 25;
const uint8_t VGA_COLOR_WHITE_ON_BLACK = 0x0F;

// Inicialização estática do Singleton
Console& Console::get_instance() {
    static Console instance; // Inicializada na primeira chamada (Thread-safe em C++11+)
    return instance;
}

Console::Console() 
    : vga_buffer(nullptr), cursor_x(0), cursor_y(0) 
{}

void Console::initialize(uintptr_t vga_addr) {
    this->vga_buffer = reinterpret_cast<uint8_t*>(vga_addr);
    this->cursor_x = 0;
    this->cursor_y = 0;
    
    // Limpar o buffer VGA
    for (size_t y = 0; y < VGA_HEIGHT; ++y) {
        for (size_t x = 0; x < VGA_WIDTH; ++x) {
            const size_t index = 2 * (y * VGA_WIDTH + x);
            this->vga_buffer[index] = ' '; // Caractere
            this->vga_buffer[index + 1] = VGA_COLOR_WHITE_ON_BLACK; // Cor
        }
    }
    LOG_INFO("Console VGA/Serial inicializado.");
}

void Console::advance_line() {
    this->cursor_x = 0;
    this->cursor_y++;
    
    if (this->cursor_y >= VGA_HEIGHT) {
        // Rolagem: Mover todas as linhas para cima (exceto a primeira)
        // Usar memmove para copiar blocos de memória
        size_t move_size = 2 * VGA_WIDTH * (VGA_HEIGHT - 1);
        memmove(this->vga_buffer, this->vga_buffer + (2 * VGA_WIDTH), move_size);
        
        // Limpar a última linha
        size_t last_line_start = 2 * VGA_WIDTH * (VGA_HEIGHT - 1);
        memset(this->vga_buffer + last_line_start, 0, 2 * VGA_WIDTH);
        
        this->cursor_y = VGA_HEIGHT - 1;
    }
}

void Console::print(const char* str) {
    if (!this->vga_buffer) return;

    for (const char* p = str; *p != '\0'; ++p) {
        if (*p == '\n') {
            this->advance_line();
        } else {
            if (this->cursor_x >= VGA_WIDTH) {
                this->advance_line();
            }

            const size_t index = 2 * (this->cursor_y * VGA_WIDTH + this->cursor_x);
            
            // # SAFETY: Escrita direta no MMIO (VGA)
            // Acesso a memória mapeada (volatile)
            this->vga_buffer[index] = *p;
            this->vga_buffer[index + 1] = VGA_COLOR_WHITE_ON_BLACK;

            this->cursor_x++;
        }
    }
}

// Lógica de log: adiciona um prefixo e chama print()
void Console::log(LogLevel level, const char* file, int line, const char* message) {
    const char* prefix = "";
    switch (level) {
        case LOG_LEVEL_INFO:    prefix = "[INFO] "; break;
        case LOG_LEVEL_WARN:    prefix = "[WARN] "; break;
        case LOG_LEVEL_ERROR:   prefix = "[ERROR]"; break;
        case LOG_LEVEL_DEBUG:   prefix = "[DEBUG]"; break;
        default: break;
    }

    // A string final seria formatada dinamicamente, mas por simplicidade:
    this->print(prefix);
    // Exemplo: this->print(" ("); this->print(file); this->print(":"); /* print line num */ this->print(") ");
    this->print(message);
    this->print("\n");
}

// ------------------------------------------------------------------------
// --- Implementação FFI para Rust (lightos_c_log) ---
// ------------------------------------------------------------------------

// A string Rust não é nula-terminada, então precisamos de uma cópia temporária ou
// de um loop direto (o loop é mais seguro e no_std).
void lightos_c_log(uint32_t severity, const uint8_t* message_ptr, size_t len) {
    Console& console = Console::get_instance();
    LogLevel level = static_cast<LogLevel>(severity);
    
    const char* prefix = "";
    switch (level) {
        case LOG_LEVEL_INFO:    prefix = "[R-INFO] "; break;
        case LOG_LEVEL_WARN:    prefix = "[R-WARN] "; break;
        case LOG_LEVEL_ERROR:   prefix = "[R-ERROR]"; break;
        default: break;
    }
    console.print(prefix);

    // Itera sobre o slice de bytes Rust
    for (size_t i = 0; i < len; ++i) {
        char c = (char)message_ptr[i];
        
        // Simplesmente escreve o caractere (sem prefixo/newline aqui)
        // # SAFETY: Escrever diretamente no buffer do VGA.
        if (c == '\n') {
            console.advance_line();
        } else {
            if (console.cursor_x >= VGA_WIDTH) {
                console.advance_line();
            }
            
            const size_t index = 2 * (console.cursor_y * VGA_WIDTH + console.cursor_x);
            
            console.vga_buffer[index] = c;
            console.vga_buffer[index + 1] = VGA_COLOR_WHITE_ON_BLACK;
            console.cursor_x++;
        }
    }
    console.print("\n"); // Adicionar newline após a mensagem Rust
}
