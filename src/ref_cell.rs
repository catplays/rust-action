

use std::cell::RefCell;


/**
外部可变性（exterior mutability）:
用 let mut 显式地声明一个可变的值，或者，用 &mut 声明一个可变引用时，编译器可以在编译时进行严格地检查，
保证只有可变的值或者可变的引用，才能修改值内部的数据.
外部可变性通过 mut 关键字声明.

内部可见性：
对并未声明成 mut 的值或者引用，也想进行修改。也就是说，在编译器的眼里，值是只读的，但是在运行时，
这个值可以得到可变借用，从而修改内部的数据，这就是 RefCell。
*/
fn ref_cell_call() {
    let data = RefCell::new(1);
    {
        // 获得 RefCell 内部数据的可变借用
        let mut v = data.borrow_mut();
        *v += 1;
    }
    println!("data: {:?}", data.borrow());
}