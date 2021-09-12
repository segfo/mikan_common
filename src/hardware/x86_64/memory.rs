extern "sysv64" {
    pub fn set_cr0(cr4: u64);
    pub fn get_cr0() -> u64;
    pub fn set_cr3(cr3: u64);
    pub fn get_cr3() -> u64;
    pub fn set_cr4(cr4: u64);
    pub fn get_cr4() -> u64;
}

// pub fn disable_paging() {
//     unsafe {
//         let cr0 = get_cr0();
//         let pg_disable_cr0 = cr0 & 0x7fff_ffff;
//         set_cr0(pg_disable_cr0);
//     }
// }
// pub fn enable_paging(pml4_page:u64){
//     unsafe{
//         let cr0 = get_cr0();
//         let pg_disable_cr0 = cr0 | 0x8000_0000;
//         set_cr0(pg_disable_cr0);
//     }
// }
