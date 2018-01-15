#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;

use std::sync::Arc;
use std::sync::Mutex;

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::web;

#[test]
fn tree() {

//     use pipeline::Pipeline;
//     use element::Element;
//     use bintree::{BinTree, BinNode};

//     let mut tree = BinTree::new();
//     tree.insert(1);
//     tree.insert(2);
//     tree.insert(3);
//     tree.insert(4);
//     tree.insert(5);
//     tree.insert(6);
//     tree.insert(7);
//     tree.insert(8);
//     tree.insert(9);
//     tree.insert(10);
//     tree.insert(11);
//     tree.insert(12);
//     tree.insert(13);
//     tree.insert(14);
//     tree.insert(15);
//     tree.insert(16);
//     tree.insert(17);
//     tree.insert(18);
//     tree.insert(19);
//     tree.insert(20);

// //     println!("{:#?}", tree);
// //     println!("{}", tree);

//     tree.pretty_print();
}

