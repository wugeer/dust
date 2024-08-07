use crate::display_node::DisplayNode;
use crate::node::FileTime;
use crate::node::Node;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ExtensionNode<'a> {
    size: u64,
    filetime: Option<u64>,
    extension: Option<&'a OsStr>,
}

pub fn get_all_file_types(
    top_level_nodes: &[Node],
    n: usize,
    by_filetime: &Option<FileTime>,
) -> Option<DisplayNode> {
    let ext_nodes = {
        let mut extension_cumulative_sizes = HashMap::new();
        build_by_all_file_types(top_level_nodes, &mut extension_cumulative_sizes);

        let mut extension_cumulative_sizes: Vec<ExtensionNode<'_>> = extension_cumulative_sizes
            .iter()
            .map(|(&extension, (size, filetime))| ExtensionNode {
                extension,
                size: *size,
                filetime: *filetime,
            })
            .collect();

        extension_cumulative_sizes.sort_by(|lhs, rhs| lhs.cmp(rhs).reverse());

        extension_cumulative_sizes
    };

    let mut ext_nodes_iter = ext_nodes.iter();

    // First, collect the first N - 1 nodes...
    let mut displayed: Vec<DisplayNode> = ext_nodes_iter
        .by_ref()
        .take(if n > 1 { n - 1 } else { 1 })
        .map(|node| DisplayNode {
            name: PathBuf::from(
                node.extension
                    .map(|ext| format!(".{}", ext.to_string_lossy()))
                    .unwrap_or_else(|| "(no extension)".to_owned()),
            ),
            size: node.size,
            filetime: node.filetime,
            children: vec![],
        })
        .collect();

    // ...then, aggregate the remaining nodes (if any) into a single  "(others)" node
    if ext_nodes_iter.len() > 0 {
        let filetime = if by_filetime.is_some() {
            ext_nodes_iter
                .clone()
                .map(|node| node.filetime)
                .max()
                .unwrap_or(Some(0))
        } else {
            None
        };
        displayed.push(DisplayNode {
            name: PathBuf::from("(others)"),
            size: ext_nodes_iter.map(|node| node.size).sum(),
            filetime,
            children: vec![],
        });
    }

    let filetime = if by_filetime.is_some() {
        displayed
            .iter()
            .map(|node| node.filetime)
            .max()
            .unwrap_or(Some(0))
    } else {
        None
    };

    let result = DisplayNode {
        name: PathBuf::from("(total)"),
        size: displayed.iter().map(|node| node.size).sum(),
        filetime,
        children: displayed,
    };

    Some(result)
}

fn build_by_all_file_types<'a>(
    top_level_nodes: &'a [Node],
    counter: &mut HashMap<Option<&'a OsStr>, (u64, Option<u64>)>,
) {
    for node in top_level_nodes {
        if node.name.is_file() {
            let ext = node.name.extension();
            let (cumulative_size, filetime) = counter.entry(ext).or_default();
            *cumulative_size += node.size;
            if node.filetime.is_some() {
                *filetime = match (&filetime, node.filetime) {
                    (Some(a_val), Some(b_val)) => Some(std::cmp::max(*a_val, b_val)),
                    (None, Some(b_val)) => Some(b_val),
                    (Some(a_val), None) => Some(*a_val),
                    (None, None) => None,
                };
            }
        }
        build_by_all_file_types(&node.children, counter)
    }
}
