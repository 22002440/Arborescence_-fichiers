use std::{path::Path, ffi::OsStr};
use crate::{file_tree::{FileTree, EntryNode}, size::Size};


impl FileTree {

    /// Display the entire file tree rooted at the specified path.
    pub fn show(&self) {
        self.show_recursive(self.get_root(), 0);
    }
    

    /// Display the file tree rooted at the specified path, sorted lexicographically
    pub fn show_lexicographic(&self){
        self.show_lexicographic_recursive(self.get_root(), 0);
    }
    
    /// Display the file tree rooted at the specified path, applying a filter if provided.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter string to apply.
    /// * `lexicographic_sort` - A flag indicating whether to sort lexicographically.
    pub fn show_filtered(&self, filter: &str, lexicographic_sort: bool) {
        if lexicographic_sort {
            self.show_lexicographic_filtered_recursive(self.get_root(), filter, 0);
        } else {
            self.show_filtered_recursive(self.get_root(), filter, 0);
        }
    }


    /// Display the entire file tree rooted at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The root path of the file tree.
    /// * `depth` - The depth of the current recursive call.
    fn show_recursive(&self, path: &Path, depth: usize) {     
            if let Some(entry_node) = self.get_map_option(path) {
                match entry_node {
                    EntryNode::File(size) => {
                        self.print_node(path, size, depth);
                    },
                    EntryNode::Directory(children) => {
                        self.print_node(path, &self.get_size(path).unwrap_or(Size::new(0)), depth);

                        for child_path in children {
                            self.show_recursive(child_path, depth + 1);
                        }
                    },
                }
            }
        }


    /// Display the file tree rooted at the specified path, sorted lexicographically.
    ///
    /// # Arguments
    ///
    /// * `path` - The root path of the file tree.
    /// * `depth` - The depth of the current recursive call.
    fn show_lexicographic_recursive(&self, path: &Path, depth: usize) {     
        if let Some(entry_node) = self.get_map_option(path) {
            match entry_node {
                EntryNode::File(size) => {
                    self.print_node(path, size, depth);
                },
                EntryNode::Directory(children) => {
                    // Triage par taille
                    let mut sorted_children: Vec<_> = children.into_iter().collect();
                    sorted_children.sort_by(|a, b| {
                        self.get_size(b).unwrap_or(Size::new(0)).cmp(&self.get_size(a).unwrap_or(Size::new(0)))
                    });

                    self.print_node(path, &self.get_size(path).unwrap_or(Size::new(0)), depth);

                    for child_path in sorted_children {
                        self.show_lexicographic_recursive(child_path, depth + 1);
                    }
                },
            }
        }
    }

    /// Display the file tree rooted at the specified path, applying a filter.
    ///
    /// # Arguments
    ///
    /// * `path` - The root path of the file tree.
    /// * `filter` - The filter string to apply.
    /// * `depth` - The depth of the current recursive call.
    fn show_filtered_recursive(&self, path: &Path, filter: &str, depth: usize) {
        if let Some(entry_node) = self.get_map_option(path) {
            match entry_node {
                EntryNode::File(size) if path.extension() == Some(OsStr::new(filter)) => {
                    self.print_node(path, size, depth);
                }
                EntryNode::Directory(children) => {
                    self.print_node(path, &self.get_size(path).unwrap_or(Size::new(0)), depth);

                    for child_path in children {
                        self.show_filtered_recursive(child_path, filter, depth + 1);
                    }
                }
                _ => {}
            }
        }
    }


    /// Display the file tree rooted at the specified path, sorted lexicographically, and filtered.
    ///
    /// # Arguments
    ///
    /// * `path` - The root path of the file tree.
    /// * `filter` - The filter string to apply.
    /// * `depth` - The depth of the current recursive call.
    pub fn show_lexicographic_filtered_recursive(&self, path: &Path, filter: &str, depth: usize) {
        if let Some(entry_node) = self.get_map_option(path) {
            match entry_node {
                EntryNode::File(size) if path.extension() == Some(OsStr::new(filter)) => {
                    self.print_node(path, size, depth);
                }
                EntryNode::Directory(children) => {
                    // Triage lexicographique
                    let mut sorted_children: Vec<_> = children.into_iter().collect();
                    sorted_children.sort();

                    self.print_node(path, &self.get_size(path).unwrap_or(Size::new(0)), depth);

                    for child_path in sorted_children {
                        self.show_lexicographic_filtered_recursive(child_path, filter, depth + 1);
                    }
                }
                _ => {}
            }
        }
    }



    fn print_node(&self, path: &Path, size: &Size, depth: usize) {
        let indent = "      ".repeat(depth);
        println!("{}{}  /{}",indent, size, path.display());
    }
}