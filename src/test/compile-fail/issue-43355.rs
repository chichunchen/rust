// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub trait Trait1<X> {
    type Output;
}

pub trait Trait2<X> {}

pub struct A;

impl<X, T> Trait1<X> for T where T: Trait2<X> {
    type Output = ();
}

impl<X> Trait1<Box<X>> for A {
//~^ ERROR conflicting implementations of trait
//~| downstream crates may implement trait `Trait2<std::boxed::Box<_>>` for type `A`
    type Output = i32;
}

fn main() {}
