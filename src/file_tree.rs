use crypto::digest::Digest;
use crypto::md5::Md5;
use hex::encode;
use crate::size::Size;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::{self, Read};
use rayon::prelude::*;

/// Represents a file or directory entry in the file tree.

#[derive(Debug)]
pub struct FileTree {
    root: PathBuf,
    map: HashMap<PathBuf, EntryNode>,
    signature: HashMap<PathBuf, String>
}

/// Represents the size of a file or directory in the file tree.

#[derive(Clone, Debug)]
pub enum EntryNode {
    File(Size),
    Directory(Vec<PathBuf>),
}

/// Represents an iterator over the paths in the file tree.

#[derive(Debug)] 
pub struct FileTreeIterator<'a> {
    iter: Iter<'a, PathBuf, EntryNode>,
}

/// Implementation of the iterator for `FileTreeIterator`

impl<'a> Iterator for FileTreeIterator<'a> {
    type Item = &'a PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(path, _)| path)
    }
}

/// Creates a new `FileTree` instance rooted at the specified path.
///
/// # Arguments
///
/// * `root` - The root path for the file tree.
///
/// # Returns
///
/// A `Result` containing the `FileTree` instance or an `std::io::Error`.

impl FileTree {
    pub fn new(root: &Path) -> std::io::Result<Self> {
        let mut map = HashMap::new();
        let mut signature= HashMap::new();
        let root_entry = FileTree::file_explorer(root, &mut map, &mut signature)?;
        map.insert(root.to_path_buf(), root_entry.clone());
        Ok(FileTree { root: root.to_path_buf(), map, signature })
        
    }

    
/// Recursively explores a directory and builds the corresponding file tree structure.
///
///# Arguments
///
/// * `path` - The path to explore.
/// * `map` - The map to store file tree entries.
/// * `signatures` - The map to store file signatures.
///
/// # Returns
///
/// An `io::Result` containing the `EntryNode` for the specified path.
///


    fn file_explorer(path: &Path, map: &mut HashMap<PathBuf, EntryNode>, signatures: &mut HashMap<PathBuf, String>) -> std::io::Result<EntryNode> {
        let metadata = fs::metadata(path)?;

        if metadata.is_file() {

            let signature = Self::calculate_signature(path)?;
            map.insert(path.to_path_buf(), EntryNode::File(Size::new(metadata.len())));
            signatures.insert(path.to_path_buf(), signature);
            Ok(EntryNode::File(Size::new(metadata.len())))

        } else if metadata.is_dir() {
            let mut children = Vec::new();
            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path().clone();
                let entry_node = FileTree::file_explorer(&entry_path, map, signatures)?;
                map.insert(entry_path.clone(), entry_node.clone());
                children.push(entry_path);
            }
            Ok(EntryNode::Directory(children))

        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Type de fichier non pris en charge",
            ))
        }
    }

/// Calculates the MD5 signature of a file.
///
/// # Arguments
///
/// * `path` - The path of the file.
///
/// # Returns
///
/// An `io::Result` containing the MD5 signature as a hexadecimal string.

    fn calculate_signature(path: &std::path::Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Md5::new();
    
        let mut buffer = [0; 8192];
    
        loop {
            let bytes_read = file.read(&mut buffer)?;
    
            if bytes_read == 0 {
                break;
            }
    
            hasher.input(&buffer[..bytes_read]);
        }
    
        let hex_hash = encode(hasher.result_str().as_bytes());
    
        Ok(hex_hash)
    }

/// Finds and returns a map of duplicate files in the file tree based on their signatures.
///
/// # Returns
///
/// A `HashMap` where each key is a signature and the corresponding value is a vector
/// containing paths of files with that signature.
/// 

pub fn find_duplicates(&self) -> HashMap<String, Vec<PathBuf>> {
    // Clone the signatures for parallel processing
    let signatures: HashMap<_, _> = self.signature.clone();

    // Create a parallel iterator over the cloned signatures
    let signature_map: HashMap<String, Vec<PathBuf>> = signatures
        .par_iter()
        .map(|(path, signature)| (path.clone(), signature.clone()))
        .fold(
            || HashMap::new(),
            |mut acc, (path, signature)| {
                acc.entry(signature).or_insert_with(Vec::new).push(path);
                acc
            },
        )
        .reduce(HashMap::new, |mut acc1, acc2| {
            for (key, mut value) in acc2 {
                acc1.entry(key).or_insert_with(Vec::new).append(&mut value);
            }
            acc1
        });

    signature_map.into_iter().filter(|(_, paths)| paths.len() > 1).collect()
}


/// Returns the root path of the file tree.

    pub fn get_root(&self) -> &Path {
        &self.root
    }

/// Returns the children (sub-paths) of a directory in the file tree.
///
/// # Arguments
///
/// * `path` - The path of the directory.
///
/// # Returns
///
/// An `Option` containing a slice of `PathBuf` representing the children of the directory.


    pub fn get_children(&self, path: &Path) -> Option<&[PathBuf]> {

        if let Some(entry_node) = self.map.get(path){
            if let EntryNode::Directory(enfants) = entry_node{
                println!("\n{:?}\n",Some(&enfants));
                return Some(&enfants);
            }           
        }
        None
    }

/// Returns the total size of a file or directory in the file tree.
///
/// # Arguments
///
/// * `path` - The path of the file or directory.
///
/// # Returns
///
/// An `Option` containing the total size as a `Size` instance.

    pub fn get_size(&self, path: &Path) -> Option<Size> {


        self.map.get(path).and_then(|entry| match entry {
            EntryNode::File(size) => Some(*size),
            EntryNode::Directory(enfants) => {
                let total_size: u64 = enfants
                .iter().filter_map(|child| self.get_size(child).map(|size| size.value())).sum();
                Some(Size::new(total_size))
            },
        })
    }

/// Returns an iterator over the paths of files in the file tree.

    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.map.iter().filter_map(|(path, entry)| {
            if let EntryNode::File(_) = entry {
                Some(path)
            } else {
                None
            }
        })
    }
    
/// Returns the entry node for a given path in the file tree.
///
/// # Arguments
///
/// * `path` - The path to retrieve the entry node for.
///
/// # Returns
///
/// An `Option` containing the `EntryNode` for the specified path.
    
    pub fn get_map_option(&self, path: &Path) -> Option<&EntryNode> {
        self.map.get(path)
    }
}






#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn file_tree_integrity_test() {
        let path = Path::new("Test");
        let result = FileTree::new(&path);
        // Vérifier si la création de FileTree réussit
        assert!(result.is_ok());
    }
    
    #[test]
    fn get_root_test() {
        // Spécifiez la racine que vous attendez
        let expected_root = Path::new("Test");
        
        // Créez un FileTree avec la racine spécifiée
        let result = FileTree::new(&expected_root);
    
        // Vérifiez si la création de FileTree réussit
        assert!(result.is_ok());
    
        // Déballer le résultat
        let file_tree = result.unwrap();
    
        // Vérifiez si la racine retournée par get_root correspond à la racine spécifiée
        assert_eq!(file_tree.get_root(), expected_root);
    }
    
    #[test]
    fn get_children_test(){
    
        // Déballer le résultat
        let path = &PathBuf::from("Test");
        let result = FileTree::new(&path);
        let file_tree = result.unwrap();
    
        // Vérifier la présence de certains répertoires
        let root_children = file_tree.get_children(path).expect("Failed to get children for the root directory");
        assert!(root_children.contains(&PathBuf::from("Test/Dossier1")));
        assert!(root_children.contains(&PathBuf::from("Test/Dossier2")));
    }
    #[test]
    fn get_children_dossier_1_test(){

        let path = &PathBuf::from("Test");
        let result = FileTree::new(&path);
        let file_tree = result.unwrap();

        // Vérifier la présence de certains fichiers et sous-dossiers dans Dossier 1
        let dossier1_children = file_tree.get_children(&PathBuf::from("Test/Dossier1")).expect("Failed to get children for Dossier 1");
        println!("{:?}", dossier1_children);
        assert!(dossier1_children.contains(&PathBuf::from("Test/Dossier1/Fichier1")));
    }

    #[test]

    fn get_children_dossier_2_test(){

        let path = &PathBuf::from("Test");
        let result = FileTree::new(&path);
        let file_tree = result.unwrap();

        // Vérifier la présence de certains fichiers, sous-dossiers dans Dossier 2
        let dossier2_children = file_tree.get_children(&PathBuf::from("Test/Dossier2")).expect("Failed to get children for Dossier 2");
        assert!(dossier2_children.contains(&PathBuf::from("Test/Dossier2/SousDossier1")));
        assert!(dossier2_children.contains(&PathBuf::from("Test/Dossier2/SousDossier2")));
        assert!(dossier2_children.contains(&PathBuf::from("Test/Dossier2/Fichier3")));
        
    }

    #[test]

    fn get_children_sous_dossier_2(){

        let path = &PathBuf::from("Test");
        let result = FileTree::new(&path);
        let file_tree = result.unwrap();

        // Vérifier la présence de certains fichiers dans Sous Dossier 2
        let sousdossier2_children = file_tree.get_children(&PathBuf::from("Test/Dossier2/SousDossier2")).expect("Failed to get children for Sous Dossier 2");
        assert!(sousdossier2_children.contains(&PathBuf::from("Test/Dossier2/SousDossier2/Fichier2")));  
    }
    
    #[test]
    fn get_size_file() {
        let metadata = fs::metadata(Path::new("Test/Dossier1/Fichier1"));
        match metadata {
            Ok(data) => {
                let path = &PathBuf::from("Test");
                let result = FileTree::new(&path);
                let file_tree = result.unwrap();
                let size = file_tree.get_size(Path::new("Test/Dossier1/Fichier1"));
                assert_eq!(size, Some(Size::new(data.len() as u64))); 
            }
            Err(err) => {
                eprintln!("Erreur lors de la récupération des métadonnées : {}", err);
                assert!(false)
            }
        }
        
    }

    #[test]
    fn get_size_directory() {
        let metadata = fs::metadata(Path::new("Test/Dossier2"));
        match metadata {
            Ok(_data) => {
                let path = &PathBuf::from("Test");
                let result = FileTree::new(&path);
                let file_tree = result.unwrap();
                let size = file_tree.get_size(Path::new("Test/Dossier2"));
                assert_eq!(size, Some(Size::new(520256)));  //Insérer la taille du dossier avant chaque essais car il change tout le temps
                                                        //data.len() ne fonctionne pas ici car il retourne l'espace occuper sur le disque
            }
            Err(err) => {
                eprintln!("Erreur lors de la récupération des métadonnées : {}", err);
                assert!(false);
            }
        }
    }

    #[test]
    fn get_size_nonexistent_path() {
        let path = &PathBuf::from("Test/Chemin/Inexistant");
        let result = FileTree::new(&path);
        
        match result {
            Ok(_file_tree) => {
                assert!(false)
            }
            Err(_err) => {
                assert!(true);
            }
        }
    }
    
    #[test]
    fn test_files() {
        // Create a sample file tree
        let mut file_tree = FileTree::new(Path::new("TestFile")).expect("Failed to create file tree");

        // Add some file entries to the file tree
        file_tree.map.insert(PathBuf::from("TestFile/file1"), EntryNode::File(Size::new(100)));
        file_tree.map.insert(PathBuf::from("TestFile/file2"), EntryNode::File(Size::new(200)));
        file_tree.map.insert(PathBuf::from("TestFile/dir1"), EntryNode::Directory(vec![PathBuf::from("TestFile/dir1/file3")]));

        // Some target that should be found in the directory
        let target_file_1= Path::new("TestFile/file1");
        let target_file_2= Path::new("TestFile/file2");
        let target_directory_1 = Path::new("TestFile/file1");

        // Get the files from the file tree
        let files = file_tree.files();
        let mut buff:Vec<&Path> = Vec::new();
        for entry in files{
            buff.push(entry);
            println!("{:?}", buff);
        }
        //Assert that the files vector contains the correct paths

        assert!(buff.contains(&target_file_1));
        assert!(buff.contains(&target_file_2));
        assert!(buff.contains(&target_directory_1));
       
        
}
    
}