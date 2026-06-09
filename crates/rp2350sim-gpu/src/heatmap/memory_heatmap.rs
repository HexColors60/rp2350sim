#![allow(dead_code)]

//! Memory heatmap visualization.

/// Memory heatmap for visualizing memory access patterns.
#[derive(Debug)]
pub struct MemoryHeatmap {
    /// Memory size.
    size: usize,
    /// Access counts per region.
    access_counts: Vec<u32>,
    /// Write counts per region.
    write_counts: Vec<u32>,
    /// Region size in bytes.
    region_size: usize,
    /// Width for rendering.
    width: f32,
    /// Height for rendering.
    height: f32,
    /// Maximum access count (for color scaling).
    max_count: u32,
}

impl MemoryHeatmap {
    /// Create a new memory heatmap.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: 520 * 1024, // 520 KB SRAM
            access_counts: vec![0; 1024],
            write_counts: vec![0; 1024],
            region_size: 512,
            width,
            height,
            max_count: 1,
        }
    }

    /// Create with custom memory size.
    pub fn with_size(memory_size: usize, region_size: usize, width: f32, height: f32) -> Self {
        let regions = (memory_size + region_size - 1) / region_size;
        Self {
            size: memory_size,
            access_counts: vec![0; regions],
            write_counts: vec![0; regions],
            region_size,
            width,
            height,
            max_count: 1,
        }
    }

    /// Record a read access at the given address.
    pub fn record_read(&mut self, addr: usize) {
        let region = addr / self.region_size;
        if region < self.access_counts.len() {
            self.access_counts[region] = self.access_counts[region].saturating_add(1);
            self.max_count = self.max_count.max(self.access_counts[region]);
        }
    }

    /// Record a write access at the given address.
    pub fn record_write(&mut self, addr: usize) {
        let region = addr / self.region_size;
        if region < self.write_counts.len() {
            self.write_counts[region] = self.write_counts[region].saturating_add(1);
            self.access_counts[region] = self.access_counts[region].saturating_add(1);
            self.max_count = self.max_count.max(self.access_counts[region]);
        }
    }

    /// Record an access at the given address.
    pub fn record_access(&mut self, addr: usize, is_write: bool) {
        if is_write {
            self.record_write(addr);
        } else {
            self.record_read(addr);
        }
    }

    /// Get the access count for a region.
    pub fn get_access_count(&self, region: usize) -> u32 {
        self.access_counts.get(region).copied().unwrap_or(0)
    }

    /// Get the write count for a region.
    pub fn get_write_count(&self, region: usize) -> u32 {
        self.write_counts.get(region).copied().unwrap_or(0)
    }

    /// Get the color for a region based on access count.
    pub fn get_color(&self, region: usize) -> [f32; 4] {
        let count = self.get_access_count(region);
        if count == 0 {
            return [0.1, 0.1, 0.1, 1.0];
        }
        
        let intensity = (count as f32 / self.max_count as f32).min(1.0);
        let writes = self.get_write_count(region) as f32 / count as f32;
        
        // Blue for reads, red for writes, gradient in between
        let r = intensity * writes;
        let g = intensity * 0.2;
        let b = intensity * (1.0 - writes);
        
        [r, g, b, 1.0]
    }

    /// Clear all access counts.
    pub fn clear(&mut self) {
        for count in &mut self.access_counts {
            *count = 0;
        }
        for count in &mut self.write_counts {
            *count = 0;
        }
        self.max_count = 1;
    }

    /// Render the heatmap.
    pub fn render(&self) {
        // Calculate grid dimensions
        let total_regions = self.access_counts.len();
        let aspect = self.width / self.height;
        let cols = ((total_regions as f32 * aspect).sqrt()) as usize;
        let rows = (total_regions + cols - 1) / cols;
        
        let cell_width = self.width / cols as f32;
        let cell_height = self.height / rows as f32;
        
        for (i, _) in self.access_counts.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * cell_width;
            let y = row as f32 * cell_height;
            let color = self.get_color(i);
            
            // Cell rendering would use macroquad or wgpu
            // For now, this is a placeholder
            let _ = (x, y, cell_width, cell_height, color);
        }
    }

    /// Resize the heatmap.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Get the number of regions.
    pub fn region_count(&self) -> usize {
        self.access_counts.len()
    }

    /// Get the region size.
    pub fn region_size(&self) -> usize {
        self.region_size
    }

    /// Get the total memory size.
    pub fn memory_size(&self) -> usize {
        self.size
    }
}