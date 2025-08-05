#[cfg(test)]
mod context_tests {
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    
    use sniff_check::utils::FileUtils;
    
    #[test]
    fn test_file_walking_with_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("test.ts"), "console.log('test');").unwrap();
        fs::write(temp_path.join("test.js"), "console.log('test');").unwrap();
        fs::write(temp_path.join("test.py"), "print('test')").unwrap();
        fs::write(temp_path.join("README.md"), "# Test").unwrap();
        
        let extensions = ["ts", "js"];
        let files = FileUtils::find_files_with_extensions(temp_path, &extensions);
        
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "test.ts"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "test.js"));
    }
    
    #[test]
    fn test_has_extension() {
        assert!(FileUtils::has_extension(Path::new("test.ts"), &["ts", "tsx"]));
        assert!(FileUtils::has_extension(Path::new("test.tsx"), &["ts", "tsx"]));
        assert!(!FileUtils::has_extension(Path::new("test.py"), &["ts", "tsx"]));
        assert!(!FileUtils::has_extension(Path::new("test"), &["ts", "tsx"]));
    }
    
    #[test]
    fn test_count_lines_optimized() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.ts");
        
        let content = "line 1\nline 2\nline 3\n";
        fs::write(&temp_file, content).unwrap();
        
        let line_count = FileUtils::count_lines_optimized(&temp_file).unwrap();
        assert_eq!(line_count, 3);
    }
    
    #[test]
    fn test_get_relative_path() {
        let current_dir = std::env::current_dir().unwrap();
        let test_path = current_dir.join("src").join("main.rs");
        
        let relative = FileUtils::get_relative_path(&test_path);
        assert!(relative.contains("src/main.rs") || relative.contains("src\\main.rs"));
    }
}