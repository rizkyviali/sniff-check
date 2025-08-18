/// Performance optimization utilities
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rayon::prelude::*;
use walkdir::WalkDir;

/// Optimized file walker with parallel processing and smart filtering
pub struct OptimizedFileWalker {
    max_depth: Option<usize>,
    follow_links: bool,
    excluded_dirs: Vec<String>,
    excluded_extensions: Vec<String>,
    parallel_threshold: usize,
}

impl OptimizedFileWalker {
    pub fn new() -> Self {
        Self {
            max_depth: None,
            follow_links: false,
            excluded_dirs: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                ".next".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "target".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
            ],
            excluded_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "ico".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "ttf".to_string(),
                "eot".to_string(),
                "mp4".to_string(),
                "mp3".to_string(),
                "webm".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
            parallel_threshold: 50, // Use parallel processing if more than 50 files
        }
    }
    
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    
    pub fn exclude_dirs<S: Into<String>>(mut self, dirs: Vec<S>) -> Self {
        self.excluded_dirs.extend(dirs.into_iter().map(|s| s.into()));
        self
    }
    
    pub fn exclude_extensions<S: Into<String>>(mut self, exts: Vec<S>) -> Self {
        self.excluded_extensions.extend(exts.into_iter().map(|s| s.into()));
        self
    }
    
    pub fn parallel_threshold(mut self, threshold: usize) -> Self {
        self.parallel_threshold = threshold;
        self
    }
    
    /// Walk directory and collect files with optimized filtering
    pub fn walk<P: AsRef<Path>>(&self, start_dir: P) -> Vec<PathBuf> {
        let mut walker = WalkDir::new(start_dir).follow_links(self.follow_links);
        
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }
        
        let files: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| self.should_include_file(entry.path()))
            .map(|entry| entry.into_path())
            .collect();
        
        files
    }
    
    /// Walk directory and collect files with specific extensions
    pub fn walk_with_extensions<P: AsRef<Path>>(&self, start_dir: P, extensions: &[&str]) -> Vec<PathBuf> {
        let mut walker = WalkDir::new(start_dir).follow_links(self.follow_links);
        
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }
        
        let files: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| self.should_include_file(entry.path()))
            .filter(|entry| self.has_extension(entry.path(), extensions))
            .map(|entry| entry.into_path())
            .collect();
        
        files
    }
    
    /// Process files in parallel if above threshold
    pub fn process_files_parallel<T, F, R>(&self, files: &[PathBuf], processor: F) -> Vec<R>
    where
        F: Fn(&PathBuf) -> T + Sync + Send,
        T: Into<R> + Send,
        R: Send,
    {
        if files.len() >= self.parallel_threshold {
            files.par_iter().map(|file| processor(file).into()).collect()
        } else {
            files.iter().map(|file| processor(file).into()).collect()
        }
    }
    
    fn should_include_file(&self, path: &Path) -> bool {
        // Check if any parent directory is excluded
        for component in path.components() {
            if let Some(dir_name) = component.as_os_str().to_str() {
                if self.excluded_dirs.contains(&dir_name.to_string()) {
                    return false;
                }
            }
        }
        
        // Check if file extension is excluded
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if self.excluded_extensions.contains(&ext_str.to_lowercase()) {
                    return false;
                }
            }
        }
        
        true
    }
    
    fn has_extension(&self, path: &Path, extensions: &[&str]) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str.to_lowercase().as_str());
            }
        }
        false
    }
}

impl Default for OptimizedFileWalker {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached file content reader for repeated access patterns
pub struct CachedFileReader {
    cache: std::collections::HashMap<PathBuf, Arc<String>>,
    cache_size_limit: usize,
    max_file_size: u64,
}

impl CachedFileReader {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            cache_size_limit: 100, // Cache up to 100 files
            max_file_size: 1024 * 1024, // Don't cache files larger than 1MB
        }
    }
    
    pub fn with_cache_limit(mut self, limit: usize) -> Self {
        self.cache_size_limit = limit;
        self
    }
    
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }
    
    /// Read file with caching
    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<Arc<String>> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Check cache first
        if let Some(content) = self.cache.get(&path_buf) {
            return Ok(content.clone());
        }
        
        // Check file size before reading
        let metadata = std::fs::metadata(&path_buf)?;
        if metadata.len() > self.max_file_size {
            // Don't cache large files, read directly
            let content = std::fs::read_to_string(&path_buf)?;
            return Ok(Arc::new(content));
        }
        
        // Read and cache the file
        let content = std::fs::read_to_string(&path_buf)?;
        let arc_content = Arc::new(content);
        
        // Manage cache size
        if self.cache.len() >= self.cache_size_limit {
            // Remove oldest entry (simple FIFO, could be improved with LRU)
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }
        
        self.cache.insert(path_buf, arc_content.clone());
        Ok(arc_content)
    }
    
    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache_size_limit)
    }
}

impl Default for CachedFileReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized line counting using memory mapping for large files
pub fn count_lines_optimized<P: AsRef<Path>>(path: P) -> std::io::Result<usize> {
    use memmap2::Mmap;
    
    let file = std::fs::File::open(&path)?;
    let metadata = file.metadata()?;
    
    // For small files, use regular reading
    if metadata.len() < 1024 * 1024 { // 1MB threshold
        let content = std::fs::read_to_string(&path)?;
        return Ok(content.lines().count());
    }
    
    // For large files, use memory mapping
    let mmap = unsafe { Mmap::map(&file)? };
    let count = mmap.par_chunks(8192) // Process in 8KB chunks
        .map(|chunk| {
            chunk.iter().filter(|&&byte| byte == b'\n').count()
        })
        .sum();
    
    Ok(count)
}

/// Batch processing utilities for better performance
pub struct BatchProcessor<T> {
    batch_size: usize,
    items: Vec<T>,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            items: Vec::new(),
        }
    }
    
    pub fn add(&mut self, item: T) -> Option<Vec<T>> {
        self.items.push(item);
        
        if self.items.len() >= self.batch_size {
            Some(std::mem::take(&mut self.items))
        } else {
            None
        }
    }
    
    pub fn flush(&mut self) -> Vec<T> {
        std::mem::take(&mut self.items)
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    checkpoints: Vec<(String, std::time::Duration)>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            checkpoints: Vec::new(),
        }
    }
    
    pub fn checkpoint(&mut self, name: &str) {
        let elapsed = self.start_time.elapsed();
        self.checkpoints.push((name.to_string(), elapsed));
    }
    
    pub fn total_elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    pub fn get_checkpoints(&self) -> &[(String, std::time::Duration)] {
        &self.checkpoints
    }
    
    pub fn print_report(&self) {
        println!("Performance Report:");
        println!("Total time: {:?}", self.total_elapsed());
        
        let mut last_time = std::time::Duration::ZERO;
        for (name, total_time) in &self.checkpoints {
            let delta = *total_time - last_time;
            println!("  {}: {:?} (Δ {:?})", name, total_time, delta);
            last_time = *total_time;
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}