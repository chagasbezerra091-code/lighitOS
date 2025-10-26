// src/kernel/interrupts/pic.rs

use x86_64::instructions::port::Port;
use spin::Mutex;
use super::{PIC_1_OFFSET, PIC_2_OFFSET};
use crate::RustKernelConfig::arch_hal::{PIC_MASTER_COMMAND_PORT, PIC_MASTER_DATA_PORT, PIC_SLAVE_COMMAND_PORT, PIC_SLAVE_DATA_PORT};

/// 🔌 Controlador Programável de Interrupções (PIC) 8259A.
/// Usado em sistemas x86_64 legados para multiplexar IRQs.
pub struct Pic {
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    /// 🏭 Cria uma nova instância do PIC (Mestre ou Escravo).
    pub const unsafe fn new(command_port: u16, data_port: u16) -> Self {
        Pic {
            command: Port::new(command_port),
            data: Port::new(data_port),
        }
    }

    /// ⚙️ Inicializa o PIC (sequência de ICW - Initialization Control Words).
    /// * Esta sequência remapeia as IRQs para que não haja conflito com as exceções da CPU.
    pub unsafe fn initialize(&mut self, offset: u8) {
        // ICW1: Inicia a inicialização
        const ICW1_INIT: u8 = 0x11;
        self.command.write(ICW1_INIT);

        // ICW2: Offset (Vetor de Interrupção)
        self.data.write(offset);

        // ICW3: Comunicação Master/Slave
        const ICW3_SLAVE_IRQ2: u8 = 0b0000_0100; // O slave está conectado ao IRQ 2 do master
        self.data.write(ICW3_SLAVE_IRQ2);

        // ICW4: Modo 8086
        const ICW4_8086_MODE: u8 = 0x01;
        self.data.write(ICW4_8086_MODE);
    }

    /// 📢 Envia um sinal EOI (End of Interrupt) para o PIC.
    /// * Essencial para que o PIC possa aceitar a próxima interrupção.
    pub unsafe fn send_eoi(&mut self) {
        const PIC_EOI: u8 = 0x20;
        self.command.write(PIC_EOI);
    }
}

/// 💾 Estrutura que representa o PIC Mestre e o PIC Escravo.
pub struct PicChained {
    master: Mutex<Pic>,
    slave: Mutex<Pic>,
}

/// 🔑 A instância global de acesso ao PIC (Mestre e Escravo).
/// Usamos Mutex para garantir que apenas um handler de interrupção possa
/// acessar as portas I/O do PIC por vez.
lazy_static! {
    pub static ref PICS: Mutex<PicChained> = Mutex::new(unsafe {
        PicChained {
            master: Mutex::new(Pic::new(PIC_MASTER_COMMAND_PORT, PIC_MASTER_DATA_PORT)),
            slave: Mutex::new(Pic::new(PIC_SLAVE_COMMAND_PORT, PIC_SLAVE_DATA_PORT)),
        }
    });
}

impl PicChained {
    /// ⚙️ Inicializa os PICs Mestre e Escravo e os remapeia.
    pub unsafe fn initialize(&mut self) {
        let mut master = self.master.lock();
        let mut slave = self.slave.lock();

        // Salvar a máscara atual antes da inicialização
        let master_mask = master.data.read();
        let slave_mask = slave.data.read();
        
        // Pausa (Pode ser simulada com instruções 'outb' extras)
        
        // Inicializa (remapeia)
        master.initialize(PIC_1_OFFSET);
        slave.initialize(PIC_2_OFFSET);

        // Restaurar a máscara de interrupção (habilitar todas as IRQs por enquanto)
        master.data.write(0x00); // Habilita todas as IRQs do Master
        slave.data.write(0x00);  // Habilita todas as IRQs do Slave
        
        // O kernel real restauraria: master.data.write(master_mask); slave.data.write(slave_mask);
    }

    /// 📢 Envia o EOI.
    /// * Se a interrupção veio do Escravo (IRQ 8-15), o EOI deve ser enviado para o Escravo E o Mestre.
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if interrupt_id >= PIC_2_OFFSET {
            self.slave.lock().send_eoi();
        }
        self.master.lock().send_eoi();
    }
}
