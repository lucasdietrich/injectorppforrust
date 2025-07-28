#![instruction_set(arm::t32)]
use crate::injector_core::common::*;
use crate::injector_core::patch_trait::*;

pub(crate) struct PatchArmv7Thumb;

impl PatchTrait for PatchArmv7Thumb {
    fn replace_function_with_other_function(
        src: FuncPtrInternal,
        target: FuncPtrInternal,
    ) -> PatchGuard {
        let patch_size = 4;
        let original_bytes = unsafe { read_bytes(src.as_ptr() as *mut u8, patch_size) };
        let jit_size = 9; // round to 10 ? 12 ?
        let jit_memory = allocate_jit_memory(&src, jit_size);
        generate_will_execute_jit_code_abs(jit_memory, target.as_ptr());
        let func_addr = src.as_ptr() as usize;
        let jit_addr = jit_memory as usize;
        let offset = (jit_addr as isize - func_addr as isize) / 2;
        let offset = offset - 2; // As PC is 2 instructions ahead in ARMv7
        
        // ARMv7 Thumb (16MB)
        if !(-33554432..=33554431).contains(&offset) {
            panic!("JIT memory is out of branch range");
        }
  
        let offset = offset as u32;

        // B (Encoding T4) always : I1 = NOT(J1 EOR S); I2 = NOT(J2 EOR S); imm32 = SignExtend(S:I1:I2:imm10:imm11:’0’, 32);
        let s = ((offset as u32) >> 23) & 1;
        let i1 = ((offset as u32) >> 22) & 1;
        let i2 = ((offset as u32) >> 21) & 1;
        let imm10 = (((offset as u32) >> 11) & 0x3ff) as u16;
        let imm11 = ((offset as u32) & 0x7ff) as u16;

        // I1 = NOT(J1 EOR S) => J1 = S EOR NOT(I1) 
        let j1 = (s ^ !i1) & 1;
        // I2 = NOT(J2 EOR S) => J2 = S EOR NOT(I2)
        let j2 = (s ^ !i2) & 1;

        let branch_instr_first: u16 = 0xf000 | ((s as u16) << 10) | imm10;
        let branch_instr_second: u16 = 0x9000 | ((j1 as u16) << 13) | ((j2 as u16) << 11) | imm11;

        let mut patch = [0u8; 4];
        patch[0..2].copy_from_slice(&branch_instr_first.to_le_bytes());
        patch[2..4].copy_from_slice(&branch_instr_second.to_le_bytes());

        unsafe {
            // src is a thumb function, so we need to patch the instruction before it
            let ptr = src.as_ptr() as usize;
            if (ptr & 1) == 0 {
                panic!("src is a ARM function, expected a Thumb function");
            }

            patch_function((ptr - 1) as *mut u8, &patch);
        }

        PatchGuard::new(
            src.as_ptr() as *mut u8,
            original_bytes,
            patch_size,
            jit_memory,
            jit_size,
        )
    }

    fn replace_function_return_boolean(src: FuncPtrInternal, value: bool) -> PatchGuard {
        todo!()
    }
}

/// The generated instructions are:
///  - `MOVW X9, #imm0` (clears the rest)
///   - `MOVT X9, #imm1, LSL #16`
///   - `MOVK X9, #imm2, LSL #32`
///   - `BR X9`
fn generate_will_execute_jit_code_abs(jit_ptr: *mut u8, target: *const ()) {
    let target_addr = target as usize as u32;

    // // nop
    // let nop: u16 = 0xbf00;
    
    // x9
    let register_name = 9;
    
    // Encoding T3
    // movw r9, #imm0 (clears the rest)
    let addr_lower_hf = (target_addr & 0xffff) as u16;
    let imm4 = (addr_lower_hf >> 12) & 0xf;
    let i = (addr_lower_hf >> 11) & 0x1;
    let imm3 = (addr_lower_hf >> 8) & 0x7;
    let imm8 = addr_lower_hf & 0xff;

    let movw_first: u16 = 0xf240 | (i << 10) | imm4;
    let movw_second: u16 = (imm3 << 12) | (register_name << 8) | imm8;

    // Encoding T1
    // movt r9, #imm1, lsl #16
    let addr_upper_hf = (target_addr >> 16) as u16;
    let imm4 = (addr_upper_hf >> 12) & 0xf;
    let i = (addr_upper_hf >> 11) & 0x1;
    let imm3 = (addr_upper_hf >> 8) & 0x7;
    let imm8 = addr_upper_hf & 0xff;

    let movt_first: u16 = 0xf2C0 | (i << 10) | imm4;
    let movt_second: u16 = (imm3 << 12) | (register_name << 8) | imm8;


    // Encoding T1
    // BR x9
    let br: u16 = 0x4700 | (register_name << 3);

    // Write instructions in the correct order: bottom-up so no overwrite
    let mut asm_code: Vec<u8> = Vec::new();
    append_instruction(&mut asm_code, movw_first);
    append_instruction(&mut asm_code, movw_second);
    append_instruction(&mut asm_code, movt_first);
    append_instruction(&mut asm_code, movt_second);
    append_instruction(&mut asm_code, br);
    

    unsafe {
        inject_asm_code(&asm_code, jit_ptr);
    }
}

fn append_instruction(asm_code: &mut Vec<u8>, instruction: u16) {
    asm_code.push((instruction & 0xFF) as u8);
    asm_code.push(((instruction >> 8) & 0xFF) as u8);
}
