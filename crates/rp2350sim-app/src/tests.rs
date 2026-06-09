//! Tests for RP2350 simulator.

use rp2350sim_cpu_arm::thumb::{decode, disassemble, ThumbKind};
use rp2350sim_cpu_hazard3::rv32::{decode as rv32_decode, disassemble as rv32_disassemble, Rv32Kind};

#[cfg(test)]
mod thumb_tests {
    use super::*;

    #[test]
    fn test_decode_mov_imm() {
        // MOVS R0, #0x42
        let opcode = 0x2242;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::MovImm);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_decode_add_reg() {
        // ADDS R0, R1, R2
        let opcode = 0x1888;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::AddReg);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.rm, 2);
    }

    #[test]
    fn test_decode_sub_reg() {
        // SUBS R0, R1, R2
        let opcode = 0x1A88;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::SubReg);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.rm, 2);
    }

    #[test]
    fn test_decode_lsl() {
        // LSLS R0, R1, #4
        let opcode = 0x0108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Lsl);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_lsr() {
        // LSRS R0, R1, #4
        let opcode = 0x0908;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Lsr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_asr() {
        // ASRS R0, R1, #4
        let opcode = 0x1108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Asr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_and() {
        // ANDS R0, R1
        let opcode = 0x4008;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::And);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_eor() {
        // EORS R0, R1
        let opcode = 0x4048;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Eor);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_orr() {
        // ORRS R0, R1
        let opcode = 0x4308;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Orr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_push() {
        // PUSH {R4, R5, LR}
        let opcode = 0xB470;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Push);
    }

    #[test]
    fn test_decode_pop() {
        // POP {R4, R5, PC}
        let opcode = 0xBC70;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Pop);
    }

    #[test]
    fn test_decode_b() {
        // B #0x100
        let opcode = 0xE040;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::B);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_beq() {
        // BEQ #0x10
        let opcode = 0xD008;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x0); // EQ
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bne() {
        // BNE #0x10
        let opcode = 0xD108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x1); // NE
    }

    #[test]
    fn test_decode_cmp_imm() {
        // CMP R0, #0x42
        let opcode = 0x2842;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::CmpImm);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_disassemble_mov() {
        let opcode = 0x2242;
        let result = disassemble(opcode, 0);
        assert!(result.contains("movs"));
    }

    #[test]
    fn test_disassemble_add() {
        let opcode = 0x1888;
        let result = disassemble(opcode, 0);
        assert!(result.contains("adds"));
    }
}

#[cfg(test)]
mod rv32_tests {
    use super::*;

    #[test]
    fn test_decode_lui() {
        // LUI x1, 0x12345
        let opcode = 0x123455B7;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lui);
        assert_eq!(instr.rd, 1);
    }

    #[test]
    fn test_decode_auipc() {
        // AUIPC x1, 0x12345
        let opcode = 0x12345597;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Auipc);
        assert_eq!(instr.rd, 1);
    }

    #[test]
    fn test_decode_jal() {
        // JAL x1, 0x1000
        let opcode = 0x001005EF;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jal);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_jalr() {
        // JALR x1, x2, 0
        let opcode = 0x000100E7;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jalr);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_addi() {
        // ADDI x1, x2, 42
        let opcode = 0x02A10093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Addi);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.imm, 42);
    }

    #[test]
    fn test_decode_add() {
        // ADD x1, x2, x3
        let opcode = 0x003100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Add);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.rs2, 3);
    }

    #[test]
    fn test_decode_sub() {
        // SUB x1, x2, x3
        let opcode = 0x403100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sub);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.rs2, 3);
    }

    #[test]
    fn test_decode_and() {
        // AND x1, x2, x3
        let opcode = 0x003170B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::And);
    }

    #[test]
    fn test_decode_or() {
        // OR x1, x2, x3
        let opcode = 0x003160B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Or);
    }

    #[test]
    fn test_decode_xor() {
        // XOR x1, x2, x3
        let opcode = 0x003140B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Xor);
    }

    #[test]
    fn test_decode_sll() {
        // SLL x1, x2, x3
        let opcode = 0x003110B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sll);
    }

    #[test]
    fn test_decode_srl() {
        // SRL x1, x2, x3
        let opcode = 0x003150B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Srl);
    }

    #[test]
    fn test_decode_sra() {
        // SRA x1, x2, x3
        let opcode = 0x403150B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sra);
    }

    #[test]
    fn test_decode_slt() {
        // SLT x1, x2, x3
        let opcode = 0x003120B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Slt);
    }

    #[test]
    fn test_decode_sltu() {
        // SLTU x1, x2, x3
        let opcode = 0x003130B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sltu);
    }

    #[test]
    fn test_decode_beq() {
        // BEQ x1, x2, 0x10
        let opcode = 0x00208663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Beq);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bne() {
        // BNE x1, x2, 0x10
        let opcode = 0x00214663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Bne);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_lw() {
        // LW x1, 0(x2)
        let opcode = 0x00012083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lw);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_sw() {
        // SW x1, 0(x2)
        let opcode = 0x00112023;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sw);
        assert!(instr.writes_mem);
    }

    #[test]
    fn test_decode_mul() {
        // MUL x1, x2, x3
        let opcode = 0x023100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Mul);
    }

    #[test]
    fn test_decode_div() {
        // DIV x1, x2, x3
        let opcode = 0x023140B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Div);
    }

    #[test]
    fn test_disassemble_lui() {
        let opcode = 0x123455B7;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("lui"));
    }

    #[test]
    fn test_disassemble_addi() {
        let opcode = 0x02A10093;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("addi"));
    }
}

#[cfg(test)]
mod gpio_tests {
    use rp2350sim_devices::gpio::{Gpio, GpioFunction, GPIO_BASE};

    #[test]
    fn test_gpio_creation() {
        let gpio = Gpio::new();
        assert_eq!(gpio.pin_count(), 48);
    }

    #[test]
    fn test_gpio_direction() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        assert!(gpio.get_pin(0).unwrap().direction);
    }

    #[test]
    fn test_gpio_output() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        gpio.set_output(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_input() {
        let mut gpio = Gpio::new();
        gpio.set_input(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_function() {
        let mut gpio = Gpio::new();
        gpio.get_pin_mut(0).unwrap().function = GpioFunction::Uart;
        assert_eq!(gpio.get_pin(0).unwrap().function, GpioFunction::Uart);
    }
}

#[cfg(test)]
mod uart_tests {
    use rp2350sim_devices::uart::{Uart, UART0_BASE, cr, regs};

    #[test]
    fn test_uart_creation() {
        let uart = Uart::uart0();
        assert_eq!(uart.base(), UART0_BASE);
    }

    #[test]
    fn test_uart_enable() {
        let mut uart = Uart::uart0();
        assert!(!uart.is_enabled());
        uart.cr = cr::UARTEN;
        assert!(uart.is_enabled());
    }

    #[test]
    fn test_uart_tx() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::TXE;
        uart.write_byte(0x41);
        assert_eq!(uart.tx_fifo.len(), 1);
    }

    #[test]
    fn test_uart_rx() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::RXE;
        uart.push_rx(0x42);
        assert_eq!(uart.rx_fifo.len(), 1);
        let byte = uart.read_byte();
        assert_eq!(byte, Some(0x42));
    }
}

#[cfg(test)]
mod timer_tests {
    use rp2350sim_devices::timer::Timer;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert_eq!(timer.get_time(), 0);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = Timer::new();
        timer.tick();
        assert_eq!(timer.get_time(), 1);
    }

    #[test]
    fn test_timer_advance() {
        let mut timer = Timer::new();
        timer.advance(100);
        assert_eq!(timer.get_time(), 100);
    }

    #[test]
    fn test_timer_alarm() {
        let mut timer = Timer::new();
        timer.set_alarm(0, 100);
        assert!(timer.is_armed(0));
        timer.advance(100);
        assert!(!timer.is_armed(0));
        assert!(timer.has_interrupt());
    }
}