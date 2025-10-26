// src/kernel/interrupts/irq_handlers_asm.s

// ------------------------------------------------------------------------
// --- Importações FFI (Rust) ---
// ------------------------------------------------------------------------
.extern lightos_timer_handler_rust
.extern lightos_keyboard_handler_rust

// ------------------------------------------------------------------------
// --- lightos_timer_handler (Ponto de Entrada Assembly do Timer IRQ) ---
// ------------------------------------------------------------------------

.global lightos_timer_handler
lightos_timer_handler:
    // 1. Salvar Contexto da Interrupção (Registradores Caller-Saved)
    // Os registradores Callee-Saved (RBP, RBX, R12-R15) são salvos pelo Assembly
    // de troca de contexto (context_switch.s). Aqui, salvamos o resto.
    
    pushq %rdi      // RDI é usado para passar argumentos
    pushq %rsi
    pushq %rdx
    pushq %rcx
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    
    // 2. Chamar o Scheduler (Função Rust)
    
    // O RDI deve conter o ponteiro para o TaskContext salvo.
    // Como o contexto de IRQ está na stack, e a stack é o contexto da Task, 
    // precisamos de uma lógica mais complexa, mas para simplificar:
    
    // Neste ponto, o RSP aponta para os registradores salvos na stack.
    // Em um kernel real, o Agendador manteria o RSP do contexto anterior.
    // Usaremos a convenção de que a função Rust manipulará o contexto salvo.
    
    // Chamada simplificada (Passando o ponteiro para o TaskContext. O Assembly de troca
    // de contexto em 'context_switch.s' faria este passo de forma mais limpa).
    
    // Se o Assembly de troca for usado, o código seria:
    // call lightos_context_switch_save // Salva o contexto na TaskContext
    // mov %rdi, %r8                    // Move o ponteiro para o TaskContext (arg 1 para o Rust)
    // mov %r8, %rdi
    // call lightos_timer_handler_rust  // Chamada ao Rust (que agenda o próximo)
    // call lightos_context_switch_restore // Restaura o contexto da nova Task (não retorna!)
    
    // Usando apenas o handler Rust, a chamada é:
    mov %rsp, %rdi  // Passar o ponteiro do RSP (contexto salvo) como Arg 1
    call lightos_timer_handler_rust

    // 3. Restaurar Contexto (Se o Rust retornou, significa que a tarefa não foi trocada)
    
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rcx
    popq %rdx
    popq %rsi
    popq %rdi

    // 4. IRETQ (Retorno da Interrupção)
    iretq 


// ------------------------------------------------------------------------
// --- lightos_keyboard_handler (Ponto de Entrada Assembly do Keyboard IRQ) ---
// ------------------------------------------------------------------------

.global lightos_keyboard_handler
lightos_keyboard_handler:
    // Salvar registradores voláteis
    pushq %rdi
    pushq %rsi
    pushq %rdx
    pushq %rcx
    
    call lightos_keyboard_handler_rust // Chama o handler Rust

    // Restaurar registradores
    popq %rcx
    popq %rdx
    popq %rsi
    popq %rdi
    
    // Retorno da Interrupção
    iretq
