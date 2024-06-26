#![no_std]

extern crate alloc;
#[macro_use]
extern crate platform;
use arch::activate_paging_mode;
use config::FRAME_BITS;
use heap::HeapAllocator;
use platform::config::HEAP_SIZE;
pub mod data;
mod frame;
mod heap;
mod manager;
mod vmm;

pub use frame::{alloc_frame_trackers, alloc_frames, free_frames, FrameTracker, VmmPageAllocator};

pub use vmm::{kernel_pgd, kernel_satp, kernel_space, map_region_to_kernel, query_kernel_space};

pub use manager::FRAME_REF_MANAGER;
#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

#[cfg(any(feature = "talloc", feature = "buddy"))]
static mut KERNEL_HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

extern "C" {
    fn ekernel();
}

pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(ekernel as usize, memory_end);
        println!("Frame allocator init success");
        #[cfg(feature = "initrd")]
        data::relocate_removable_data();
        #[cfg(any(feature = "talloc", feature = "buddy"))]
        HEAP_ALLOCATOR.init(unsafe { &mut KERNEL_HEAP });
        #[cfg(feature = "talloc")]
        println!("Talloc allocator init success");
        #[cfg(feature = "slab")]
        println!("Slab allocator init success");
        #[cfg(feature = "buddy")]
        println!("Buddy allocator init success");
        vmm::build_kernel_address_space(memory_end);
        println!("Build kernel address space success");
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
        println!("Activate paging mode success");
    } else {
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
    }
}
