#![no_std]

extern crate alloc;
use alloc::{string::String as AllocString, vec::Vec as AllocVec, format};

// Re-export commonly used types
pub use alloc::{string::String, vec::Vec, boxed::Box};
pub use core::{result::Result as CoreResult, option::Option};

// Error handling
pub mod error {
    use super::*;
    
    pub type Result<T> = CoreResult<T, AidokuError>;

    #[derive(Debug, Clone)]
    pub struct AidokuError {
        pub message: String,
    }

    impl AidokuError {
        pub fn new(message: &str) -> Self {
            Self {
                message: message.into(),
            }
        }
    }
}

pub use error::{Result, AidokuError};

// Basic manga data structures
#[derive(Debug, Clone, Default)]
pub struct Manga {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub cover: Option<String>,
    pub categories: Option<Vec<String>>,
    pub status: MangaStatus,
    pub nsfw: MangaContentRating,
    pub viewer: MangaViewer,
}

#[derive(Debug, Clone, Default)]
pub struct Chapter {
    pub id: String,
    pub title: Option<String>,
    pub volume: Option<f32>,
    pub chapter: Option<f32>,
    pub date_updated: Option<f64>,
    pub scanlator: Option<String>,
    pub url: Option<String>,
    pub lang: String,
}

#[derive(Debug, Clone, Default)]
pub struct Page {
    pub content: PageContent,
    pub has_description: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PageContent {
    Url(String),
    Raw(Vec<u8>),
}

impl Default for PageContent {
    fn default() -> Self {
        PageContent::Url(String::new())
    }
}

impl PageContent {
    pub fn url(url: String) -> Self {
        PageContent::Url(url)
    }
    
    pub fn raw(data: Vec<u8>) -> Self {
        PageContent::Raw(data)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MangaPageResult {
    pub manga: Vec<Manga>,
    pub has_more: bool,
}

// Enums for manga data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MangaStatus {
    Unknown,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

impl Default for MangaStatus {
    fn default() -> Self {
        MangaStatus::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MangaContentRating {
    Safe,
    Suggestive,
    Nsfw,
}

impl Default for MangaContentRating {
    fn default() -> Self {
        MangaContentRating::Safe
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MangaViewer {
    Rtl,
    Ltr,
    Vertical,
    Scroll,
}

impl Default for MangaViewer {
    fn default() -> Self {
        MangaViewer::Rtl
    }
}

// Filter system
#[derive(Debug, Clone)]
pub enum Filter {
    Text { id: String, value: String },
    Select { id: String, value: i32 },
    Sort { id: String, value: String, ascending: bool },
    Check { id: String, value: bool },
    Group { id: String, filters: Vec<Filter> },
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Text,
    Select,
    Sort,
    Check,
    Group,
}

#[derive(Debug, Clone)]
pub struct Listing {
    pub name: String,
}

// HTTP method enum
#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

// Deep link support
#[derive(Debug, Clone)]
pub struct DeepLink {
    pub manga: Option<Manga>,
    pub chapter: Option<Chapter>,
}

// Prelude module for convenience imports
pub mod prelude {
    pub use super::{
        Manga, Chapter, Page, PageContent, MangaPageResult,
        MangaStatus, MangaContentRating, MangaViewer,
        Filter, FilterType, Listing, Result, AidokuError,
        String, Vec, Option, HttpMethod, DeepLink,
        Node, NodeSelection, NodeItem, StringRef, Request,
    };
    pub use alloc::{format, string::ToString, vec};
    pub use core::{default::Default, result::Result as CoreResult};
}

// Standard library replacements - basic types only
pub mod std {
    pub use alloc::{string::String, vec::Vec};
    pub use super::{HttpMethod, Result, AidokuError};
    
    // Placeholder for network operations - each source will implement its own
    pub mod net {
        use super::*;
        
        // These are placeholder types - actual implementations will vary by source
        pub struct Request;
        pub struct Html;
        pub struct ObjectRef;
        
        impl Request {
            pub fn new(_url: &str, _method: HttpMethod) -> Self {
                Request
            }
        }
    }
    
    pub fn current_date() -> f64 {
        // This should be replaced with actual timestamp in real implementations
        1234567890.0
    }
}

// Network types - improved implementation
#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(url: &str, method: HttpMethod) -> Self {
        Self {
            url: url.into(),
            method,
            headers: Vec::new(),
            body: None,
        }
    }
    
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
    
    pub fn body(mut self, body: &[u8]) -> Self {
        self.body = Some(body.to_vec());
        self
    }
    
    // Placeholder - actual implementation would need HTTP client
    pub fn html(self) -> Result<Node> {
        Ok(Node::new(&format!("<html><body>Mock response for {}</body></html>", self.url)))
    }
}

// HTML parsing - improved placeholder implementation
#[derive(Debug, Clone)]
pub struct Node {
    content: String,
}

impl Node {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.into(),
        }
    }
    
    pub fn select(&self, _selector: &str) -> NodeSelection {
        NodeSelection {
            nodes: Vec::new(),
            text: String::new(),
        }
    }
    
    pub fn text(&self) -> StringRef {
        StringRef(self.content.clone())
    }
    
    pub fn attr(&self, _name: &str) -> StringRef {
        StringRef(String::new())
    }
    
    pub fn html(&self) -> StringRef {
        StringRef(self.content.clone())
    }
}

#[derive(Debug, Clone)]
pub struct NodeSelection {
    nodes: Vec<Node>,
    text: String,
}

impl NodeSelection {
    pub fn text(&self) -> StringRef {
        StringRef(self.text.clone())
    }
    
    pub fn attr(&self, _name: &str) -> StringRef {
        StringRef(String::new())
    }
    
    pub fn array(self) -> Vec<NodeItem> {
        self.nodes.into_iter().map(|n| NodeItem::Node(n)).collect()
    }
    
    pub fn first(self) -> NodeSelection {
        self
    }
    
    pub fn html(&self) -> StringRef {
        StringRef(self.text.clone())
    }
}

#[derive(Debug, Clone)]
pub enum NodeItem {
    Node(Node),
}

impl NodeItem {
    pub fn as_node(self) -> Result<Node> {
        match self {
            NodeItem::Node(n) => Ok(n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringRef(pub String);

impl StringRef {
    pub fn read(&self) -> String {
        self.0.clone()
    }
    
    pub fn as_date(&self, _format: &str, _locale: Option<&str>, _timezone: Option<&str>) -> Result<f64> {
        // Placeholder - would need actual date parsing
        Ok(1234567890.0)
    }
}

impl From<&String> for StringRef {
    fn from(s: &String) -> Self {
        StringRef(s.clone())
    }
}

// Utility functions
pub fn current_date() -> f64 {
    1234567890.0 // Placeholder
}

// Global allocator and panic handler for no_std environment
use alloc::alloc::{GlobalAlloc, Layout};

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op
    }
}

#[global_allocator]
static GLOBAL: DummyAllocator = DummyAllocator;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Re-export macros
pub use aidoku_stable_macros::*;