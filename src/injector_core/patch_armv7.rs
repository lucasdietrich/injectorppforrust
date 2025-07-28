#![cfg(target_arch = "arm")]
use crate::injector_core::common::*;
use crate::injector_core::patch_trait::*;

pub(crate) struct PatchArmv7;

impl PatchTrait for PatchArmv7 {
    fn replace_function_with_other_function(
        src: FuncPtrInternal,
        target: FuncPtrInternal,
    ) -> PatchGuard {
        let patch_size = 4;
        let original_bytes = unsafe { read_bytes(src.as_ptr() as *mut u8, patch_size) };
        let jit_size = 12;
        let jit_memory = allocate_jit_memory(&src, jit_size);
        generate_will_execute_jit_code_abs(jit_memory, target.as_ptr());
        let func_addr = src.as_ptr() as usize;
        let jit_addr = jit_memory as usize;
        let offset = (jit_addr as isize - func_addr as isize) / 4;
        let offset = offset - 2; // As PC is 2 instructions ahead in ARMv7
        if !(-33554432..=33554431).contains(&offset) {
            panic!("JIT memory is out of branch range");
        }
        // B (A2 encoding) always
        let branch_instr: u32 = 0xea000000 | ((offset as u32) & 0x00ffffff);

        let mut patch = [0u8; 4];
        patch[0..4].copy_from_slice(&branch_instr.to_le_bytes());
        unsafe {
            patch_function(src.as_ptr() as *mut u8, &patch);
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
    
    // x9
    let register_name = 9;
    
    // Encoding A2
    // movw r9, #imm0 (clears the rest)
    let addr_lower_hf = (target_addr & 0xffff) as u16;
    let movw: u32 = 0xe3000000 | (((addr_lower_hf >> 12) as u32) << 16) | ((register_name as u32) << 12) | (addr_lower_hf as u32 & 0xfff);
    
    // Encoding A1
    // movt r9, #imm1, lsl #16
    let addr_upper_hf = (target_addr >> 16) as u16;
    let movt: u32 = 0xe3400000 | (((addr_upper_hf >> 12) as u32) << 16) | ((register_name as u32) << 12) | (addr_upper_hf as u32 & 0xfff);

    // Encoding A1
    // BR x9
    let br: u32 = 0xe12fff10 | (register_name as u32);

    // Write instructions in the correct order: bottom-up so no overwrite
    let mut asm_code: Vec<u8> = Vec::new();
    append_instruction(&mut asm_code, movw);
    append_instruction(&mut asm_code, movt);
    append_instruction(&mut asm_code, br);

    unsafe {
        inject_asm_code(&asm_code, jit_ptr);
    }
}

fn append_instruction(asm_code: &mut Vec<u8>, instruction: u32) {
    asm_code.push((instruction & 0xFF) as u8);
    asm_code.push(((instruction >> 8) & 0xFF) as u8);
    asm_code.push(((instruction >> 16) & 0xFF) as u8);
    asm_code.push(((instruction >> 24) & 0xFF) as u8);
}
