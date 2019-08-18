//! Nyar语言运行时环境
//!
//! 提供Nyar语言的运行时支持，包括内存管理、异步执行和标准库。

use gc_arena::{Arena, Collect, Gc, GcCell, MutationContext, Rootable};
use nyar_lir::{Value, VirtualMachine, ExecutionContext, Class, Trait, Enum};
use std::collections::HashMap;

/// Nyar运行时环境
pub struct Runtime {
    /// Tokio运行时
    tokio_runtime: tokio::runtime::Runtime,
    /// 全局变量
    globals: HashMap<String, Value<'static>>,
    /// 内存分配器
    arena: Arena<RuntimeRoots>,
    /// 标准库
    stdlib: StdLib,
}

/// 运行时根对象
#[derive(Collect)]
#[collect(no_drop)]
pub struct RuntimeRoots<'gc> {
    /// 虚拟机
    pub vm: VirtualMachine<'gc>,
    /// 全局类定义
    pub classes: HashMap<String, Gc<'gc, Class<'gc>>>,
    /// 全局特征定义
    pub traits: HashMap<String, Gc<'gc, Trait<'gc>>>,
    /// 全局枚举定义
    pub enums: HashMap<String, Gc<'gc, Enum<'gc>>>,
}

/// 标准库
pub struct StdLib {
    /// 是否已加载
    loaded: bool,
}

impl Runtime {
    /// 创建一个新的运行时环境
    pub fn new() -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
            
        let arena = Arena::new(|mc| {
            RuntimeRoots {
                vm: VirtualMachine::new(mc),
                classes: HashMap::new(),
                traits: HashMap::new(),
                enums: HashMap::new(),
            }
        });
        
        Self {
            tokio_runtime,
            globals: HashMap::new(),
            arena,
            stdlib: StdLib { loaded: false },
        }
    }
    
    /// 加载标准库
    pub fn load_stdlib(&mut self) -> crate::Result<()> {
        if self.stdlib.loaded {
            return Ok(());
        }
        
        self.arena.mutate(|mc, roots| {
            // 创建基本类型
            let object_class = Class::new(mc, "Object");
            roots.classes.insert("Object".to_string(), object_class);
            
            // 创建基本特征
            let comparable_trait = Trait::new(mc, "Comparable");
            roots.traits.insert("Comparable".to_string(), comparable_trait);
            
            // 创建基本枚举
            let option_enum = Enum::new(mc, "Option");
            roots.enums.insert("Option".to_string(), option_enum);
        });
        
        self.stdlib.loaded = true;
        Ok(())
    }
    
    /// 获取全局变量
    pub fn get_global(&self, name: &str) -> Option<Value<'static>> {
        self.globals.get(name).cloned()
    }
    
    /// 设置全局变量
    pub fn set_global(&mut self, name: &str, value: Value<'static>) {
        self.globals.insert(name.to_string(), value);
    }
    
    /// 执行异步任务
    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.tokio_runtime.block_on(future)
    }
    
    /// 创建一个新的类
    pub fn create_class(&mut self, name: &str) -> crate::Result<()> {
        self.arena.mutate(|mc, roots| {
            let class = Class::new(mc, name);
            roots.classes.insert(name.to_string(), class);
        });
        Ok(())
    }
    
    /// 创建一个新的特征
    pub fn create_trait(&mut self, name: &str) -> crate::Result<()> {
        self.arena.mutate(|mc, roots| {
            let trait_obj = Trait::new(mc, name);
            roots.traits.insert(name.to_string(), trait_obj);
        });
        Ok(())
    }
    
    /// 创建一个新的枚举
    pub fn create_enum(&mut self, name: &str) -> crate::Result<()> {
        self.arena.mutate(|mc, roots| {
            let enum_obj = Enum::new(mc, name);
            roots.enums.insert(name.to_string(), enum_obj);
        });
        Ok(())
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl StdLib {
    /// 创建一个新的标准库
    pub fn new() -> Self {
        Self { loaded: false }
    }
}