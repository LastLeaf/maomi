use std::cell::Cell;

use super::TableRow;

thread_local! {
    static ID: Cell<usize> = Cell::new(0);
}

fn gen_id() -> usize {
    ID.with(|x| {
        let ret = x.get() + 1;
        x.set(ret);
        ret
    })
}

fn random_member<'a>(list: &'a [&'static str]) -> &'a str {
    let mut n: [u8; 1] = [0];
    getrandom::getrandom(&mut n).unwrap();
    list[(n[0] as usize) % list.len()]
}

pub(crate) fn build(count: usize) -> Vec<TableRow> {
    let adjectives = [
        "pretty",
        "large",
        "big",
        "small",
        "tall",
        "short",
        "long",
        "handsome",
        "plain",
        "quaint",
        "clean",
        "elegant",
        "easy",
        "angry",
        "crazy",
        "helpful",
        "mushy",
        "odd",
        "unsightly",
        "adorable",
        "important",
        "inexpensive",
        "cheap",
        "expensive",
        "fancy",
    ];
    let colours = [
        "red",
        "yellow",
        "blue",
        "green",
        "pink",
        "brown",
        "purple",
        "brown",
        "white",
        "black",
        "orange",
    ];
    let nouns = [
        "table",
        "chair",
        "house",
        "bbq",
        "desk",
        "car",
        "pony",
        "cookie",
        "sandwich",
        "burger",
        "pizza",
        "mouse",
        "keyboard",
    ];
    let mut ret = Vec::with_capacity(count);
    for _ in 0..count {
        ret.push(TableRow {
            id: gen_id(),
            label: format!("{} {} {}", random_member(&adjectives), random_member(&colours), random_member(&nouns)),
        });
    }
    ret
}
