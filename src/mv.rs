use std::mem::{size_of, transmute};
use std::ptr::{null, null_mut};
#[derive(Copy, Clone, Debug)]

enum Opcode {
    OAddC,
    OAddD,
    OAddI,
    OAndA,
    OAndC,
    OAndD,
    OAndI,
    OCall,
    OCallext,
    OCastCD,
    OCastCI,
    OCastDC,
    OCastDI,
    OCastIC,
    OCastID,
    ODivC,
    ODivD,
    ODivI,
    ODrop,
    OEnter,
    OEqA,
    OEqC,
    OEqD,
    OEqI,
    OGreaterC,
    OGreaterD,
    OGreaterI,
    OGreaterEqC,
    OGreaterEqD,
    OGreaterEqI,
    OHalt,
    OInsert,
    OJfA,
    OJfC,
    OJfD,
    OJfI,
    OJmp,
    OJtA,
    OJtC,
    OJtD,
    OJtI,
    OLessC,
    OLessD,
    OLessI,
    OLessEqC,
    OLessEqD,
    OLessEqI,
    OLoad,
    OMulC,
    OMulD,
    OMulI,
    ONegC,
    ONegD,
    ONegI,
    ONop,
    ONotA,
    ONotC,
    ONotD,
    ONotI,
    ONotEqA,
    ONotEqC,
    ONotEqD,
    ONotEqI,
    OOffset,
    OOrA,
    OOrC,
    OOrD,
    OOrI,
    OPushFPAddr,
    OPushCtA,
    OPushCtC,
    OPushCtD,
    OPushCtI,
    ORet,
    OStore,
    OSubC,
    OSubD,
    OSubI,
}
// struct Instr{
//     opcode:
// }
const STACK_SIZE: usize = 32 * 1024;
#[derive(Debug)]
struct VirtualMachine {
    sp: *mut u8,          // stack pointer
    stack_after: *mut u8, // used for stack limit
    stack: [u8; STACK_SIZE],
}
impl VirtualMachine {
    fn new() -> Self {
        let stack = [0; STACK_SIZE];
        let sp = null_mut();
        let stack_after = null_mut();
        let mut mv = Self {
            sp,
            stack_after,
            stack,
        };
        mv.sp = mv.stack.as_mut_ptr();
        mv.stack_after = unsafe { mv.sp.add(STACK_SIZE) };
        mv
    }
    fn check_top<T: Sized>(&self) {
        if unsafe { self.sp.add(size_of::<T>()) > self.stack_after } {
            panic!("Out of stack");
        }
    }
    fn check_bot<T: Sized>(&mut self) {
        if unsafe { self.sp.sub(size_of::<T>()) < self.stack.as_mut_ptr() } {
            panic!("Not enough stack bytes");
        }
    }
    fn push<T: Sized>(&mut self, v: T) {
        self.check_top::<T>();
        let x = self.sp as *mut T;
        unsafe {
            std::ptr::write(x, v);
        }
        self.sp = unsafe { self.sp.add(size_of::<T>()) };
    }
    fn pop<T: Sized>(&mut self) -> T {
        self.check_bot::<T>();
        self.sp = unsafe { self.sp.sub(size_of::<T>()) };
        let x = self.sp as *mut T;
        let v = unsafe { std::ptr::read(x) };
        v
    }

    fn run(&mut self, instr_list: &InstrList) {
        let (mut ival1, mut ival2) = (0, 0);
        let (mut cval1, mut cval2) = (0 as char, 0 as char);
        let (mut dval1, mut dval2) = (0., 0.);
        let (mut aval1, mut aval2) = (null::<()>(), null::<()>());
        let stack_after = self.stack_after;
        let mut fp = null_mut();
        let mut ip = instr_list.front;
        loop {
            print!("{:p} | {}\t", ip, unsafe {
                self.sp.offset_from(self.stack.as_ptr())
            });
            let ipi = unsafe { *ip };
            match ipi.opcode {
                Opcode::OCall => {
                    aval1 = ipi.arg1.unwrap_or_default().get_addr();
                    println!("CALL\t{:p}", aval1);
                    self.push(ipi.next);
                }
                Opcode::OCallext => {
                    let faddr = ipi.arg1.unwrap().get_addr();
                    println!("CALLEXT\t{:p}", faddr);
                    let f2 = unsafe { transmute::<*const (), fn()>(faddr) };
                    f2();
                    ip = ipi.next;
                }
                Opcode::OCastID => {
                    ival1 = self.pop();
                    dval1 = ival1 as f64;
                    println!("CAST_I_D\t({} -> {}", ival1, dval1);
                    self.push(dval1);
                    ip = ipi.next;
                }
                Opcode::OCastIC => {
                    ival1 = self.pop();
                    cval1 = std::char::from_u32(ival1 as u32).unwrap();
                    //cval1 = unsafe { transmute::<i32, char>(ival1) };
                    println!("CAST_I_C\t{} -> {}", ival1, cval1);
                    self.push(cval1);
                    ip = ipi.next;
                }
                Opcode::OCastCD => {
                    cval1 = self.pop();
                    dval1 = cval1 as u64 as f64;
                    println!("CAST_C_D\t{} -> {}", cval1, dval1);
                    self.push(dval1);
                    ip = ipi.next;
                }
                Opcode::OCastCI => {
                    cval1 = self.pop();
                    ival1 = cval1 as i64;
                    println!("CAST_C_I\t{} -> {}", cval1, ival1);
                    self.push(ival1);
                    ip = ipi.next;
                }
                Opcode::OCastDC => {
                    dval1 = self.pop();
                    cval1 = std::char::from_u32(dval1 as u32).unwrap();
                    println!("CAST_D_C\t{} -> {}", dval1, cval1);
                    self.push(cval1);
                    ip = ipi.next;
                }
                Opcode::OCastDI => {
                    dval1 = self.pop();
                    ival1 = dval1 as i64;
                    println!("CAST_D_I\t{} -> {}", dval1, ival1);
                    self.push(ival1);
                    ip = ipi.next;
                }
                Opcode::ODrop => {
                    ival1 = ipi.arg1.unwrap().get_int();
                    println!("DROP\t{}", ival1);
                    if unsafe { self.sp.sub(ival1 as usize) } < self.stack.as_mut_ptr() {
                        panic!("Not enough stack bytes");
                    }
                    self.sp = unsafe { self.sp.sub(ival1 as usize) };
                    ip = ipi.next;
                }
                Opcode::OEnter => {
                    ival1 = ipi.arg1.unwrap().get_int();
                    println!("ENTER\t{}", ival1);
                    self.push(fp);
                    fp = self.sp;
                    self.sp = unsafe { self.sp.add(ival1 as usize) };
                    ip = ipi.next
                }
                Opcode::OEqD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("EQ_D\t{} == {} -> {}", dval1, dval1, dval1 == dval2);
                    self.push((dval1 == dval2) as u64);
                    ip = ipi.next
                }
                Opcode::OEqC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("EQ_D\t{} == {} -> {}", cval1, cval1, cval1 == cval2);
                    self.push((cval1 == cval2) as u64);
                    ip = ipi.next
                }
                Opcode::OEqA => {
                    aval1 = self.pop();
                    aval2 = self.pop();
                    println!("EQ_D\t{:p} == {:p} -> {}", aval1, aval1, aval1 == aval2);
                    self.push((aval1 == aval2) as u64);
                    ip = ipi.next
                }
                Opcode::OEqI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("EQ_D\t{} == {} -> {}", ival1, ival2, ival1 == ival2);
                    self.push((ival1 == ival2) as u64);
                    ip = ipi.next
                }
                Opcode::ONotEqD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("NOTEQ_D\t{} != {} -> {}", dval1, dval2, dval1 != dval2);
                    self.push((dval1 != dval2) as u64);
                    ip = ipi.next
                }
                Opcode::ONotEqA => {
                    aval1 = self.pop();
                    aval2 = self.pop();
                    println!("NOTEQ_A\t{:p} != {:p} -> {}", aval1, aval2, aval1 != aval2);
                    self.push((aval1 != aval2) as u64);
                    ip = ipi.next
                }
                Opcode::ONotEqC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("NOTEQ_C\t{} != {} -> {}", cval1, cval2, cval1 != cval2);
                    self.push((cval1 != cval2) as u64);
                    ip = ipi.next
                }
                Opcode::ONotEqI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("NOTEQ_I\t{} != {} -> {}", ival1, ival2, ival1 != ival2);
                    self.push((ival1 != ival2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("GREATER_D\t{} > {} -> {}", dval1, dval2, dval1 > dval2);
                    self.push((dval1 > dval2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("GREATER_I\t{} > {} -> {}", ival1, ival2, ival1 > ival2);
                    self.push((ival1 > ival2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("GREATER_C\t{} > {} -> {}", cval1, cval2, cval1 > cval2);
                    self.push((cval1 > cval2) as u64);
                    ip = ipi.next
                }
                Opcode::OLessD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("LESS_D\t{} < {} -> {}", dval1, dval2, dval1 < dval2);
                    self.push((dval1 < dval2) as u64);
                    ip = ipi.next
                }
                Opcode::OLessI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("LESS_I\t{} < {} -> {}", ival1, ival2, ival1 < ival2);
                    self.push((ival1 < ival2) as u64);
                    ip = ipi.next
                }
                Opcode::OLessC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("LESS_C\t{} < {} -> {}", cval1, cval2, cval1 < cval2);
                    self.push((cval1 < cval2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterEqD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("GREATEREQ_D\t{} >= {} -> {}", dval1, dval2, dval1 >= dval2);
                    self.push((dval1 >= dval2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterEqI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("GREATEREQ_I\t{} >= {} -> {}", ival1, ival2, ival1 >= ival2);
                    self.push((ival1 >= ival2) as u64);
                    ip = ipi.next
                }
                Opcode::OGreaterEqC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("GREATEREQ_C\t{} >= {} -> {}", cval1, cval2, cval1 >= cval2);
                    self.push((cval1 >= cval2) as u64);
                    ip = ipi.next;
                }
                Opcode::OLessEqD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("LESSEQ_D\t{} <= {} -> {}", dval1, dval2, dval1 <= dval2);
                    self.push((dval1 <= dval2) as u64);
                    ip = ipi.next
                }
                Opcode::OLessEqI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("LESSEQ_I\t{} <= {} -> {}", ival1, ival2, ival1 <= ival2);
                    self.push((ival1 > ival2) as u64);
                    ip = ipi.next
                }
                Opcode::OLessEqC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!("LESSEQ_C\t{} <= {} -> {}", cval1, cval2, cval1 <= cval2);
                    self.push((cval1 <= cval2) as u64);
                    ip = ipi.next
                }
                Opcode::OHalt => {
                    println!("Halt");
                    return;
                }
                Opcode::OInsert => {
                    ival1 = ipi.arg1.unwrap().get_int(); // idst
                    ival2 = ipi.arg2.unwrap().get_int(); // nbytes
                    println!("INSERT\t{}, {}", ival1, ival2);
                    if unsafe { self.sp.add(ival2 as usize) } > stack_after {
                        panic!("Out of stack");
                    }
                    unsafe {
                        std::ptr::copy(
                            self.sp.sub(ival1 as usize),
                            self.sp.sub((ival1 + ival2) as usize),
                            ival1 as usize,
                        );
                        std::ptr::copy(
                            self.sp.add(ival2 as usize),
                            self.sp.sub(ival1 as usize),
                            ival2 as usize,
                        );
                    }
                    self.sp = unsafe { self.sp.add(ival2 as usize) };
                    ip = ipi.next;
                }
                Opcode::OJtI => {
                    ival1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JT_I\t{:p}\t{}", jaddr, ival1);
                    if ival1 != 0 {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJtA => {
                    aval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JT_A\t{:p}\t{:p}", jaddr, aval1);
                    if aval1 != null() {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJtC => {
                    cval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JT_C\t{:p}\t{}", jaddr, cval1);
                    if cval1 as u64 != 0 {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJtD => {
                    dval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JT_D\t{:p}\t{}", jaddr, dval1);
                    if dval1 != 0. {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJmp => {
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JT_D\t{:p}", jaddr);
                    ip = jaddr;
                }
                Opcode::OJfI => {
                    ival1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JF_I\t{:p}\t({})", jaddr, ival1);
                    if ival1 != 0 {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJfA => {
                    aval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JF_A\t{:p}\t({:p})", jaddr, aval1);
                    if aval1 != null() {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJfC => {
                    cval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JF_C\t{:p}\t({})", jaddr, cval1);
                    if cval1 != 0 as char {
                        ip = jaddr;
                    } else {
                        ip = ipi.next
                    }
                }
                Opcode::OJfD => {
                    dval1 = self.pop();
                    let jaddr = ipi.arg1.unwrap().get_addr() as *mut Instr;
                    println!("JF_D\t{:p}\t({})", jaddr, dval1);
                    if dval1 != 0. {
                        ip = jaddr;
                    } else {
                        ip = ipi.next;
                    }
                }
                Opcode::OOffset => {
                    ival1 = self.pop();
                    aval1 = self.pop();
                    let finaddr = unsafe { aval1.add(ival1 as usize) };
                    println!("OFFSET\t{:p} + {} = {:p}", aval1, ival1, finaddr);
                    self.push(finaddr);
                    ip = ipi.next;
                }
                Opcode::OPushFPAddr => {
                    aval1 = ipi.arg1.unwrap().get_addr();
                    let finaddr = unsafe { fp.add(ival1 as usize) };
                    println!("OFFSET\t{}\t {:p}", ival1, finaddr);
                    self.push(finaddr);
                    ip = ipi.next;
                }

                Opcode::OPushCtA => {
                    aval1 = ipi.arg1.unwrap().get_addr();
                    println!("PUSHCT_A\t{:p}", aval1);
                    self.push(aval1);
                    ip = ipi.next;
                }
                Opcode::OPushCtI => {
                    ival1 = ipi.arg1.unwrap().get_int();
                    println!("PUSHCT_I\t{}", ival1);
                    self.push(ival1);
                    ip = ipi.next;
                }
                Opcode::OPushCtC => {
                    cval1 = std::char::from_u32(ipi.arg1.unwrap().get_int() as u32).unwrap();
                    println!("PUSHCT_C\t{}", cval1);
                    self.push(cval1);
                    ip = ipi.next;
                }
                Opcode::OPushCtD => {
                    dval1 = ipi.arg1.unwrap().get_double();
                    println!("PUSHCT_I\t{}", dval1);
                    self.push(dval1);
                    ip = ipi.next;
                }
                Opcode::ORet => {
                    ival1 = ipi.arg1.unwrap().get_int();
                    ival2 = ipi.arg2.unwrap().get_int();
                    println!("RET\t{}, {}", ival1, ival2);
                    let oldsp = self.sp;
                    self.sp = fp;
                    fp = self.pop();
                    ip = self.pop();
                    if unsafe { self.sp.sub(ival1 as usize) < self.stack.as_mut_ptr() } {
                        panic!("Not enough bytes");
                    }
                    self.sp = unsafe { self.sp.sub(ival1 as usize) };
                    unsafe {
                        std::ptr::copy(oldsp.sub(ival2 as usize), self.sp, ival2 as usize);
                    }
                    self.sp = unsafe { self.sp.add(ival2 as usize) };
                }

                Opcode::OStore => {
                    ival1 = ipi.arg1.unwrap().get_int();
                    let saddr = unsafe { self.sp.sub(size_of::<*mut ()>() + ival1 as usize) };
                    if saddr < self.stack.as_mut_ptr() {
                        panic!("not enough stack bytes for SET");
                    }
                    aval1 = unsafe { *(saddr as *mut *mut ()) };
                    println!("STORE\t{}\t{:p}", ival1, aval1);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            self.sp.sub(ival1 as usize),
                            aval1 as *mut u8,
                            ival1 as usize,
                        );
                    }
                    self.sp = saddr;
                    ip = ipi.next;
                }
                Opcode::OLoad => {
                    ival1 = ipi.arg1.unwrap().get_int(); // load nbytes
                    aval1 = self.pop();
                    println!("Load\t{}\t{:p}", ival1, aval1);
                    if unsafe { self.sp.add(ival1 as usize) > stack_after } {
                        panic!("Out of stack");
                    }
                    unsafe {
                        std::ptr::copy_nonoverlapping(aval1 as *const u8, self.sp, ival1 as usize);
                    }
                    self.sp = unsafe { self.sp.add(ival1 as usize) };
                    ip = ipi.next;
                }
                Opcode::OSubD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("SUB_D\t{} - {} -> {}", dval2, dval1, dval2 - dval1);
                    self.push(dval2 - dval1);
                    ip = ipi.next;
                }
                Opcode::OSubI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("SUB_I\t{} - {} -> {}", ival2, ival1, ival2 - ival1);
                    self.push(ival2 - ival1);
                    ip = ipi.next;
                }
                Opcode::OSubC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "SUB_C\t{}  {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 - cval1 as u8) as char
                    );
                    self.push((cval2 as u8 - cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::OAddC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "Add_C\t{} + {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 + cval1 as u8) as char
                    );
                    self.push((cval2 as u8 + cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::OAddD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("Add_D\t{} + {} -> {}", dval2, dval1, dval1 + dval2);
                    self.push((cval2 as u8 + cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::OAddI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("Add_I\t{} + {} -> {}", ival2, ival1, ival2 + ival1);
                    self.push(ival2 + ival1);
                    ip = ipi.next;
                }
                Opcode::OAndC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "And_C\t{} && {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 != 0 && cval1 as u8 != 0)
                    );
                    self.push((cval2 as u8 != 0 && cval1 as u8 != 0) as i64);
                    ip = ipi.next;
                }
                Opcode::OAndD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!(
                        "And_D\t{} && {} -> {}",
                        dval2,
                        dval1,
                        dval1 != 0. && dval2 != 0.
                    );
                    self.push((dval1 != 0. && dval2 != 0.) as i64);
                    ip = ipi.next;
                }
                Opcode::OAndI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!(
                        "And_I\t{} && {} -> {}",
                        ival2,
                        ival1,
                        ival2 != 0 && ival1 != 0
                    );
                    self.push((ival1 != 0 && ival2 != 0) as i64);
                    ip = ipi.next;
                }
                Opcode::OAndA => {
                    aval1 = self.pop();
                    aval2 = self.pop();
                    println!(
                        "And_A\t{:p} && {:p} -> {}",
                        aval2,
                        aval1,
                        (!aval2.is_null() && !aval1.is_null())
                    );
                    self.push((!aval2.is_null() && !aval1.is_null()) as i64);
                    ip = ipi.next;
                }
                Opcode::OOrC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "Or_C\t{} || {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 != 0 || cval1 as u8 != 0)
                    );
                    self.push((cval2 as u8 != 0 || cval1 as u8 != 0) as i64);
                    ip = ipi.next;
                }
                Opcode::OOrD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!(
                        "And_D\t{} || {} -> {}",
                        dval2,
                        dval1,
                        dval1 != 0. || dval2 != 0.
                    );
                    self.push((dval1 != 0. || dval2 != 0.) as i64);
                    ip = ipi.next;
                }
                Opcode::OOrI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!(
                        "Or_I\t{} || {} -> {}",
                        ival2,
                        ival1,
                        ival2 != 0 || ival1 != 0
                    );
                    self.push((ival1 != 0 || ival2 != 0) as i64);
                    ip = ipi.next;
                }
                Opcode::OOrA => {
                    aval1 = self.pop();
                    aval2 = self.pop();
                    println!(
                        "Or_A\t{:p} || {:p} -> {}",
                        aval2,
                        aval1,
                        (!aval2.is_null() || !aval1.is_null())
                    );
                    self.push((!aval2.is_null() || !aval1.is_null()) as i64);
                    ip = ipi.next;
                }
                Opcode::ODivC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "Div_C\t{} / {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 / cval1 as u8) as char
                    );
                    self.push((cval2 as u8 / cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::ODivD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("Div_D\t{} / {} -> {}", dval2, dval1, dval1 / dval2);
                    self.push(dval2 / dval1);
                    ip = ipi.next;
                }
                Opcode::ODivI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("Div_I\t{} / {} -> {}", ival2, ival1, ival2 / ival1);
                    self.push(ival2 / ival1);
                    ip = ipi.next;
                }
                Opcode::OMulC => {
                    cval1 = self.pop();
                    cval2 = self.pop();
                    println!(
                        "Mul_C\t{} * {} -> {}",
                        cval2,
                        cval1,
                        (cval2 as u8 * cval1 as u8) as char
                    );
                    self.push((cval2 as u8 * cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::OMulD => {
                    dval1 = self.pop();
                    dval2 = self.pop();
                    println!("Mul_D\t{} * {} -> {}", dval2, dval1, dval1 * dval2);
                    self.push((cval2 as u8 * cval1 as u8) as char);
                    ip = ipi.next;
                }
                Opcode::OMulI => {
                    ival1 = self.pop();
                    ival2 = self.pop();
                    println!("Add_I\t{} * {} -> {}", ival2, ival1, ival2 * ival1);
                    self.push(ival2 * ival1);
                    ip = ipi.next;
                }
                Opcode::ONegC => {
                    cval1 = self.pop();
                    println!("Neg_C\t-{} -> {}", cval1, !(cval1 as u8) + 1);
                    self.push(!(cval1 as u8) + 1);
                    ip = ipi.next;
                }
                Opcode::ONegI => {
                    ival1 = self.pop();
                    println!("Neg_C\t-{} -> {}", ival1, -ival1);
                    self.push(-ival1);
                    ip = ipi.next;
                }
                Opcode::ONegD => {
                    dval1 = self.pop();
                    println!("Neg_C\t-{} -> {}", dval1, -dval1);
                    self.push(-dval1);
                    ip = ipi.next;
                }
                Opcode::ONop => {
                    println!("NOP");
                    ip = ipi.next;
                }
                Opcode::ONotC => {
                    cval1 = self.pop();
                    println!("Neg_C\t-{} -> {}", cval1, cval1 as u8 != 0);
                    self.push(cval1 as u8 != 0);
                    ip = ipi.next;
                }
                Opcode::ONotI => {
                    ival1 = self.pop();
                    println!("Neg_C\t-{} -> {}", ival1, ival1 != 0);
                    self.push(ival1 != 0);
                    ip = ipi.next;
                }
                Opcode::ONotD => {
                    dval1 = self.pop();
                    println!("Neg_C\t-{} -> {}", dval1, dval1 != 0.);
                    self.push(dval1 != 0.);
                    ip = ipi.next;
                }

                _ => {
                    panic!("Invalid opcode");
                }
            }
            //break;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum InstrArg {
    Int(i64), // int char
    Double(f64),
    Addr(*const ()),
}
impl Default for InstrArg {
    fn default() -> Self {
        InstrArg::Addr(null())
    }
}
impl InstrArg {
    pub fn get_addr(&self) -> *const () {
        if let InstrArg::Addr(addr) = self {
            return *addr;
        }
        std::ptr::null()
    }
    pub fn get_int(&self) -> i64 {
        if let InstrArg::Int(o) = self {
            return *o;
        }
        0
    }
    pub fn get_double(&self) -> f64 {
        if let InstrArg::Double(o) = self {
            return *o;
        }
        0.
    }
}

#[derive(Copy, Clone, Debug)]
struct Instr {
    opcode: Opcode,
    arg1: Option<InstrArg>,
    arg2: Option<InstrArg>,
    next: *mut Instr,
    last: *mut Instr,
}
impl Instr {
    fn new(op: Opcode) -> Self {
        Self {
            opcode: op,
            arg1: None,
            arg2: None,
            next: null_mut(),
            last: null_mut(),
        }
    }
    fn new_arg(op: Opcode, arg: InstrArg) -> Self {
        Self {
            opcode: op,
            arg1: Some(arg),
            arg2: None,
            next: null_mut(),
            last: null_mut(),
        }
    }
    fn new_arg2(op: Opcode, arg1: InstrArg, arg2: InstrArg) -> Self {
        Self {
            opcode: op,
            arg1: Some(arg1),
            arg2: Some(arg2),
            next: null_mut(),
            last: null_mut(),
        }
    }
    fn as_ptr(&self) -> *const Self {
        self as *const Self
    }
    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }
    // fn insert_after(&mut self, instr: &mut Instr) {
    //     self.next = instr.next;
    //     self.last = instr.as_ptr();
    //     instr.next = self.as_ptr();
    //     if self.next.is_null() {}
    // }
}
#[derive(Clone, Debug)]
struct InstrList {
    front: *mut Instr,
    back: *mut Instr,
}
impl InstrList {
    fn new() -> Self {
        let x = null_mut::<Instr>();
        Self { front: x, back: x }
    }
    fn push_back(&mut self, i: *mut Instr) -> *const Instr {
        // (*i).next = (*self.back).next;
        // (*i).last = self.back;
        // (*self.back).next = i;
        // self.back = i;
        // return i;

        unsafe { (*i).last = self.back };
        if !self.back.is_null() {
            unsafe {
                (*self.back).next = i;
            }
        } else {
            self.front = i;
        }
        self.back = i;
        i
    }
    fn push_back_op(&mut self, op: Opcode) -> *const Instr {
        let i = Instr::new(op).as_mut_ptr();
        self.push_back(i)
    }
    fn insert_after(&mut self, after: *mut Instr, i: *mut Instr) {
        unsafe {
            (*i).next = (*after).next;
            (*i).last = after;
            (*after).next = i;
        }
        if i.is_null() {
            self.back = i;
        }
    }
}
#[derive(Copy, Clone, Debug)]
struct InstrCursor {
    curr: *const Instr,
}
impl Iterator for InstrCursor {
    type Item = Instr;
    fn next(&mut self) -> Option<Instr> {
        if self.curr.is_null() {
            return None;
        }
        let i = self.curr;
        unsafe {
            self.curr = (*self.curr).next;
        }
        Some(unsafe { *i })
    }
}
impl IntoIterator for InstrList {
    type Item = Instr;
    type IntoIter = InstrCursor;
    fn into_iter(self) -> Self::IntoIter {
        InstrCursor { curr: self.front }
    }
}

const GLOBAL_SIZE: usize = 32 * 1024;
fn check_global_size(num_globals: usize, size: usize) {
    if num_globals + size > GLOBAL_SIZE {
        panic!("Insuficient globals space");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::mv::*;
    use crate::symbols::{add_ext_funcs, require_symbol, Context};
    #[test]
    fn mv_test() {
        let mut instr_list = InstrList::new();
        let globals: [u8; GLOBAL_SIZE] = [0; GLOBAL_SIZE];
        let _num_globals: usize = 0;
        // Init a context
        let mut contexts = vec![Context::default()];
        add_ext_funcs(&mut contexts[0]);
        // Add instructions
        instr_list.push_back(
            Instr::new_arg(
                Opcode::OPushCtA,
                InstrArg::Addr(globals.as_ptr() as *const ()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(3)).as_mut_ptr());
        instr_list.push_back(
            Instr::new_arg(Opcode::OStore, InstrArg::Int(size_of::<isize>() as i64)).as_mut_ptr(),
        );
        let l1 = instr_list.push_back(
            Instr::new_arg(
                Opcode::OPushCtA,
                InstrArg::Addr(globals.as_ptr() as *const ()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(Opcode::OLoad, InstrArg::Int(size_of::<isize>() as i64)).as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(
                Opcode::OCallext,
                InstrArg::Addr(require_symbol(&contexts, "put_i").ao.get_addr()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(
                Opcode::OPushCtA,
                InstrArg::Addr(globals.as_ptr() as *const ()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(
                Opcode::OPushCtA,
                InstrArg::Addr(globals.as_ptr() as *const ()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(Opcode::OLoad, InstrArg::Int(size_of::<isize>() as i64)).as_mut_ptr(),
        );
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(1)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OSubI).as_mut_ptr());
        instr_list.push_back(
            Instr::new_arg(Opcode::OStore, InstrArg::Int(size_of::<isize>() as i64)).as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(
                Opcode::OPushCtA,
                InstrArg::Addr(globals.as_ptr() as *const ()),
            )
            .as_mut_ptr(),
        );
        instr_list.push_back(
            Instr::new_arg(Opcode::OLoad, InstrArg::Int(size_of::<isize>() as i64)).as_mut_ptr(),
        );
        instr_list
            .push_back(Instr::new_arg(Opcode::OJtI, InstrArg::Addr(l1 as *const ())).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(10)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(5)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OSubI).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(10.)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(3.)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::ODivD).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(10.)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(3.5)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OMulD).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(65)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OCastIC).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(10.5)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtD, InstrArg::Double(3.88)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OGreaterD).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(10)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(0)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OAndI).as_mut_ptr());

        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(10)).as_mut_ptr());
        instr_list.push_back(Instr::new_arg(Opcode::OPushCtI, InstrArg::Int(1)).as_mut_ptr());
        instr_list.push_back(Instr::new(Opcode::OAndI).as_mut_ptr());

        instr_list.push_back(Instr::new(Opcode::OHalt).as_mut_ptr());

        let mut mv = VirtualMachine::new();
        mv.run(&instr_list);
    }
}
