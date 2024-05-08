use std::fmt;

/// Represents the size of a file or directory.
#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
pub struct Size(u64);

impl Size {

    /// Creates a new Size instance with the specified size in bytes.
    
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Gets the value of the size in bytes.
    
    pub fn value(&self) -> u64 {
        self.0
    }
}
 

impl fmt::Display for Size {

    /// Formats the size in a human-readable format with appropriate units (e.g., KB, MB).
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate_name::Size;
    ///
    /// let size = Size::new(2048);
    /// assert_eq!(format!("{}", size), "2 KB");
    /// ```
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        let mut file_size  = self.0 as f64;
        let mut index = 0;
        while file_size >= 1024.0 && index < units.len() - 1 {
            file_size /= 1024.0;
            index += 1;
        }

        let rounded_size = (file_size * 100.0).round() / 100.0;
        write!(f, "{} {}", rounded_size, units[index])
 
    }
}

impl std::ops::Add for Size {

    /// Adds two Size instances, returning a new Size instance with the combined size in bytes.

    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}


#[cfg(test)]
mod tests {
    use super::Size;

    #[test]
    fn add_test() {
        let file_1 = Size::new(1500);
        let file_2 = Size::new(1500);
        let total = file_1 + file_2;
        assert_eq!(total, Size::new(3000))
    }

    #[test]
    fn add_test_empty(){
        let file_1 = Size::new(1500);
        let file_2 = Size::new(0);
        let total = file_1 + file_2;
        assert_eq!(total, Size::new(1500))
    }

    #[test]
    fn add_test_une_variable(){
        let file_1 = Size::new(1500);
        let mut file_2 = Size::new(0);
        file_2 = file_2 + file_1;
        assert_eq!(file_2, Size::new(1500))
    }

    
    #[test]
    fn display_kb_test(){
        let ftd = Size:: new(1024);
        assert_eq!(format!("{ftd}"), "1 KB")
    }

    #[test]
    fn display_mb_test(){
        let ftd = Size:: new(2411724);
        assert_eq!(format!("{ftd}"), "2.3 MB")
    }
    #[test]
    fn display_gb_test(){
        let ftd = Size::new(1073741824);
        assert_eq!(format!("{ftd}"), "1 GB")
    }
}