// src/bootloader/bootloader.S

.set KERNEL_STACK_SIZE, 0x20000  // 128KB de stack

// ------------------------------------------------------------------------
// --- Cabeçalho Multiboot2 ---
// ------------------------------------------------------------------------

// O Multiboot2 exige este cabeçalho nos primeiros 8192 bytes do arquivo.
.set ALIGN,    1<<0             // Alinhar ao limite de 4KB
.set MEMINFO,  1<<1             // Forçar Multiboot a fornecer a tabela de memória
.set FLAGS,    ALIGN | MEMINFO 
.set MAGIC,    0x10000000       // Magic Number do Multiboot2 (multiboot2.h)

.global multiboot_header
.section .multiboot2
multiboot_header:
    .long MAGIC                  // Magic number
    .long ARCH_X86_64            // Arquitetura (0 para x86, 4 para x86_64)
    .long 0x00000000             // Tamanho do cabeçalho (a ser preenchido)
    .long 0x00000000             // Checksum (deve ser 0)
    
    // Tag de fim obrigatória
    .word 0x0000                 // Tipo 0 (Final)
    .word 0x0000                 // Flags (0)
    .long 0x00000008             // Tamanho (8 bytes)

// ------------------------------------------------------------------------
// --- Ponto de Entrada do Bootloader ---
// ------------------------------------------------------------------------

.global _start
.section .text
_start:
    // Ponto de entrada após o Multiboot carregar o Kernel em modo 32-bit protegido.
    
    // 1. Desabilitar Interrupções
    cli
    
    // 2. Transição para o Modo Longo (64-bit) - Lógica simplificada
    // Em um bootloader real, você precisaria:
    // a) Mapear 4KB de página temporária.
    // b) Carregar o CR3 com o endereço da tabela de páginas temporária.
    // c) Habilitar PAE (Physical Address Extension).
    // d) Habilitar o Modo Longo (setando o bit LME no EFER/MSR).
    // e) Carregar o GDT de 64 bits.
    // f) Limpar os registradores (importante para evitar lixo).
    
    // Por simplificação (usando QEMU/GRUB que fazem o básico):
    mov $0x0000000000000000, %rbp // Limpa o Base Pointer
    mov $0x0000000000000000, %rsp // Limpa o Stack Pointer (Será redefinido)
    
    // 3. Configurar a Stack Temporária
    // Definimos a stack na parte superior da memória alocada para o Kernel.
    // Assumimos que a stack temporária está em KERNEL_PHYS_START + KERNEL_STACK_SIZE
    mov $(0x100000 + KERNEL_STACK_SIZE), %rsp 

    // 4. Salvar o Endereço Multiboot
    // O endereço de info do Multiboot2 é passado em %rbx
    push %rbx
    
    // 5. Chamada Final para a Função C (c_boot_entry)
    // O endereço do Multiboot2 é passado como primeiro argumento (rdi)
    pop %rdi
    call c_boot_entry
    
    // 6. Loop Infinito (Em caso de erro ou retorno inesperado)
    cli
halt:
    hlt
    jmp halt

// ------------------------------------------------------------------------
// --- Seção BSS e Data ---
// ------------------------------------------------------------------------

.section .bss
    // Aqui iriam variáveis globais não inicializadas (como a stack do kernel)
    .align 4096
    kernel_stack:
        .skip KERNEL_STACK_SIZE
