// ベアメタルで使うなら
// mem や ptr　を
// core::mem や core::ptr に書き換える
use core::{mem, ptr};

// メモリエリアを表現する構造体（双方向リスト）
// 以下、エンティティと呼ぶ
#[derive(Clone, Copy, Debug)]
pub struct MemoryArea {
    start: usize,
    size: usize,
    prev: MemoryAreaPtr,
    next: MemoryAreaPtr,
}
impl MemoryArea {
    pub fn new() -> Self {
        MemoryArea {
            start: 0,
            size: 0,
            prev: MemoryAreaPtr(ptr::null_mut()),
            next: MemoryAreaPtr(ptr::null_mut()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MemoryAreaPtr(*mut MemoryArea);

// ページサイズ。変更しちゃダメ。
pub const PAGE_SIZE: usize = 0x1000;
pub struct PageMemory {
    mem: *mut u8,
    pages: usize,
}
impl PageMemory {
    pub fn memory(&self) -> *mut u8 {
        self.mem
    }
    pub fn pages(&self) -> usize {
        self.pages
    }
    pub fn size(&self) -> usize {
        self.pages * PAGE_SIZE
    }
}

#[derive(Debug)]
pub struct PageMemoryManager {
    unused_entity_list: MemoryAreaPtr,
    free_memarea_list: MemoryAreaPtr,
    free_total_size: usize,
    lost_total: usize,
    mem_total: usize,
    // バイアス
    // nullと競合する0スタートのアドレスを管理する場合に設定される。
    bias: usize,
}

impl PageMemoryManager {
    unsafe fn trans_ptr(area: &mut MemoryArea) -> MemoryAreaPtr {
        let ptr = mem::transmute::<&mut MemoryArea, *mut MemoryArea>(area);
        MemoryAreaPtr(ptr)
    }
    pub fn new(area: &mut [MemoryArea]) -> Self {
        unsafe {
            PageMemoryManager {
                unused_entity_list: MemoryAreaPtr(ptr::null_mut()),
                free_memarea_list: MemoryAreaPtr(ptr::null_mut()),
                free_total_size: 0,
                lost_total: 0,
                mem_total: 0,
                bias: 0,
            }
            .init_instance(area)
        }
    }
    pub fn set_addr_bias(&mut self, bias: usize) {
        self.bias = bias;
    }

    pub unsafe fn init_instance(mut self, area: &mut [MemoryArea]) -> Self {
        self.list_init(area);
        self.free_memarea_list = unsafe { self.get_entity() };
        self
    }

    pub unsafe fn free_frames(&mut self, page: PageMemory) {
        let mut list = self.free_memarea_list;
        // 内部表現：bias分下駄を履かせる。
        // 0開始のアドレスも管理したいが、
        // 仕組み上0=NULLポインタ（ptr::null()）とコンフリクトするため
        // 下駄を履かせる必要がある。
        let free_start = page.mem as usize + self.bias;
        let size = page.pages * PAGE_SIZE;
        let mut current = list;
        loop {
            if (*list.0).start > free_start {
                break;
            }
            current = list;
            list = (*list.0).next;
            if list.0 == ptr::null_mut() {
                break;
            }
        }
        let next = (*current.0).next;
        // とりあえず挿入
        let free_area = self.get_entity();
        // 管理構造体の空きがなくなってしまった。
        if free_area.0 == ptr::null_mut() {
            self.lost_total += size;
            return;
        }
        // メモリの空き合計サイズを増やす
        self.free_total_size += page.pages() * PAGE_SIZE;
        // 空きエリア情報を初期化する
        (*free_area.0).start = free_start;
        (*free_area.0).size = size;
        (*current.0).next = free_area;
        (*free_area.0).prev = current;
        (*free_area.0).next = next;
        if next.0 != ptr::null_mut() {
            (*next.0).prev = free_area;
        }

        // 結合できる場合(前)
        let prev = *(*free_area.0).prev.0;
        if prev.start + prev.size == (*free_area.0).start {
            (*free_area.0).start = (*(*free_area.0).prev.0).start;
            (*free_area.0).size += (*(*free_area.0).prev.0).size;
            let prev = (*free_area.0).prev;
            self.free_entity(prev);
        }
        // 結合できる場合(後)
        let current = *free_area.0;
        let not_null = (*free_area.0).next.0 != ptr::null_mut();

        if not_null && current.start + current.size == (*next.0).start {
            (*free_area.0).size += (*next.0).size;
            // nextを線形リストから外す
            self.free_entity(next);
        }
    }

    pub fn get_freearea_bytes(&self) -> usize {
        self.free_total_size
    }

    // 空のリストを全部つないで、空きのリストとして保持しておく
    unsafe fn list_init(&mut self, area: &mut [MemoryArea]) {
        let max = area.len() - 1;
        for i in 1..max {
            area[i].next = PageMemoryManager::trans_ptr(&mut area[i + 1]);
            area[i].prev = PageMemoryManager::trans_ptr(&mut area[i - 1]);
        }
        area[1].prev = PageMemoryManager::trans_ptr(&mut area[0]);
        area[0].next = PageMemoryManager::trans_ptr(&mut area[1]);
        area[max].prev = PageMemoryManager::trans_ptr(&mut area[max - 1]);
        self.unused_entity_list = PageMemoryManager::trans_ptr(&mut area[0]);
    }

    unsafe fn get_entity(&mut self) -> MemoryAreaPtr {
        let null = MemoryAreaPtr(ptr::null_mut());
        // 空きリストの先頭から取る
        let entity = self.unused_entity_list;
        if (*entity.0).next != null {
            let entity = self.unused_entity_list;
            // 空きリストの先頭を次に進める。
            self.unused_entity_list = (*entity.0).next;
            // entityの初期化をする
            (*entity.0).start = 0;
            (*entity.0).size = 0;
            (*entity.0).prev = MemoryAreaPtr(ptr::null_mut());
            (*(*entity.0).next.0).prev = MemoryAreaPtr(ptr::null_mut());
            (*entity.0).next = MemoryAreaPtr(ptr::null_mut());
            entity
        } else {
            null
        }
    }
    // エンティティの開放（使い終わったものを管理テーブルに戻す）
    unsafe fn free_entity(&mut self, entity: MemoryAreaPtr) {
        if (*entity.0).prev.0 == ptr::null_mut() {
            // そもそも先頭要素だった場合
            // 次の要素（あれば）のprevをnullにする。
            if (*entity.0).next.0 != ptr::null_mut() {
                (*(*entity.0).next.0).prev.0 = ptr::null_mut();
            }
            // 次の要素自体がない場合は、そのまま開放する。
        } else {
            // 先頭要素ではない
            (*(*entity.0).prev.0).next = (*entity.0).next;
            if (*entity.0).next.0 != ptr::null_mut() {
                (*(*entity.0).next.0).prev = (*entity.0).prev;
            }
        }
        // 開放処理
        (*entity.0).start = 0;
        (*entity.0).size = 0;
        // 0.|    (null)<-prev-{unused_entity_list.0}<-prev/next->{next}                  |
        // 1.|    {entity}<-prev-{unused_entity_list.0}<-prev/next->{next}                |
        (*self.unused_entity_list.0).prev = entity;
        // 2.|    {entity}<-prev/next->{unused_entity_list.0}-next->{next}                |
        (*entity.0).next = self.unused_entity_list;
        // 3.|    (null)<-prev-{entity}<-prev/next->{unused_entity_list.0}-next->{next}   |
        (*entity.0).prev = MemoryAreaPtr(ptr::null_mut());
        // 3.|    (null)<-prev-{unused_entity_list.0}<-prev/next->{next}-next->{next}     |
        self.unused_entity_list.0 = entity.0;
    }

    pub unsafe fn allocate_frames(&mut self, require_size: usize) -> Option<PageMemory> {
        let mut list = self.free_memarea_list;
        let size = (require_size + 0xfff) & !0xfff;
        loop {
            if list.0 == ptr::null_mut() || (*list.0).size >= size {
                break;
            }
            list = (*list.0).next;
        }
        if list.0 == ptr::null_mut() {
            return None;
        }
        // メモリの空き合計サイズを減らす
        self.free_total_size -= size;
        // 開始アドレスをbias分引いて実アドレスにする。
        let page = PageMemory {
            mem: mem::transmute::<usize, *mut u8>((*list.0).start - self.bias),
            pages: size / PAGE_SIZE,
        };
        if (*list.0).size - size > 0 {
            (*list.0).size -= size;
            (*list.0).start += size;
        } else {
            // 管理している領域のサイズがゼロになったら管理情報は不要なので破棄
            self.free_entity(list);
        }
        Some(page)
    }
}

// メモリアドレスとサイズからページメモリを生成するユーティリティ関数
pub fn memtranse(mem: usize, size: usize) -> PageMemory {
    PageMemory {
        mem: unsafe { mem::transmute::<usize, *mut u8>(mem) },
        pages: ((size + 0xfff) & !0xfff) / PAGE_SIZE,
    }
}

/////////////////////////////////////////////
// デバッグ用関数
// fn show_area(area: MemoryArea) -> String {
//     format!(
//         "MemoryArea {{ start: 0x{:x}, size: 0x{:x}, prev: {:?}, next: {:?} }}",
//         area.start, area.size, area.prev, area.next
//     )
// }

// fn show_list(mlist: MemoryAreaPtr) {
//     let mut list = mlist;
//     loop {
//         unsafe {
//             if list.0 == ptr::null_mut() {
//                 break;
//             }
//             println!("{:?} : {}", list, show_area(*list.0));
//             list = (*list.0).next;
//         }
//     }
// }

// pub fn show_available_memory(manager: &PageMemoryManager) {
//     let list = manager.unused_entity_list;
//     println!("===statistics start===");
//     println!("=== unused entities ===");
//     show_list(list);
//     println!("=== registered free memory area ===");
//     let list = manager.free_memarea_list;
//     show_list(list);
//     println!("===statistics end===\n");
// }
