const STD_INPUT_HANDLE: u32 = -10_i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11_i32 as u32;
const STD_ERROR_HANDLE: u32 = -12_i32 as u32;

pub fn alloc_console() -> bool {
    unsafe {
        AllocConsole() != 0
    }
}

pub fn free_console() -> bool {
    unsafe { 
        SetStdHandle(STD_INPUT_HANDLE, 0);
        SetStdHandle(STD_OUTPUT_HANDLE, 0);
        SetStdHandle(STD_ERROR_HANDLE, 0);
        FreeConsole() == 0 
    } 
}

///
/// returns true if app was started from console
pub fn is_console() -> bool {
    unsafe {
        let mut buffer = [0u32; 1];
        let count = GetConsoleProcessList(buffer.as_mut_ptr(), 1);
        count != 1
    }
}

///
/// returns true if app is a console app, not used now
pub fn has_console() -> bool {
    unsafe {
        let mut buffer = [0u32; 1];
        let count = GetConsoleProcessList(buffer.as_mut_ptr(), 1);
        count != 0
    }
}

#[link(name="Kernel32")]
extern "system" {
    fn AllocConsole() -> i32;
    fn GetConsoleProcessList(processList: *mut u32, count: u32) -> u32;
    fn FreeConsole() -> i32;
    fn SetStdHandle(nStdHandle: u32, hHandle: isize) -> i32;
}
