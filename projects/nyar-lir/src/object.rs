//! Nyar语言的对象系统
//!
//! 实现了Nyar语言的对象系统，包括Class、Trait和Enum。

use crate::value::{NyarValue, Function};
use gc_arena::{Arena, Collect, Gc, Mutation};
use std::collections::HashMap;

/// 类定义
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Class<'gc> {
    /// 类名
    pub name: Gc<'gc, String>,
    /// 父类
    pub parent: Option<Gc<'gc, Class<'gc>>>,
    /// 实现的特征
    pub traits: Vec<Gc<'gc, Trait<'gc>>>,
    /// 方法表
    pub methods: HashMap<String, Gc<'gc, Function<'gc>>>,
    /// 静态属性
    pub static_properties: HashMap<String, NyarValue<'gc>>,
}

/// 特征（接口）定义
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Trait<'gc> {
    /// 特征名
    pub name: Gc<'gc, String>,
    /// 父特征
    pub parents: Vec<Gc<'gc, Trait<'gc>>>,
    /// 方法签名
    pub method_signatures: HashMap<String, MethodSignature>,
}

/// 方法签名
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct MethodSignature {
    /// 方法名
    pub name: String,
    /// 参数类型（在动态类型系统中主要用于文档和静态分析）
    pub parameter_types: Vec<String>,
    /// 返回类型（在动态类型系统中主要用于文档和静态分析）
    pub return_type: String,
}

/// 枚举（抽象类）定义
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Enum<'gc> {
    /// 枚举名
    pub name: Gc<'gc, String>,
    /// 变体
    pub variants: HashMap<String, EnumVariant<'gc>>,
    /// 方法表
    pub methods: HashMap<String, Gc<'gc, Function<'gc>>>,
}

/// 枚举变体
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct EnumVariant<'gc> {
    /// 变体名
    pub name: String,
    /// 关联数据
    pub fields: Vec<String>,
    /// 构造函数
    pub constructor: Option<Gc<'gc, Function<'gc>>>,
}

/// 对象实例
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NyarObject<'gc> {
    /// 类引用
    pub class: Option<Gc<'gc, Class<'gc>>>,
    /// 枚举引用（如果是枚举实例）
    pub enum_type: Option<Gc<'gc, Enum<'gc>>>,
    /// 当前变体（如果是枚举实例）
    pub variant: Option<String>,
    /// 属性表
    pub properties: HashMap<String, NyarValue<'gc>>,
}

impl<'gc> Class<'gc> {
    /// 创建一个新的类
    pub fn new(mc: &Mutation<'gc>, name: &str) -> Gc<'gc, Self> {
        Gc::new(mc, Self {
            name: Gc::new(mc, name.to_string()),
            parent: None,
            traits: Vec::new(),
            methods: HashMap::new(),
            static_properties: HashMap::new(),
        })
    }

    /// 添加方法
    pub fn add_method(&mut self, name: &str, method: Gc<'gc, Function<'gc>>) {
        self.methods.insert(name.to_string(), method);
    }

    /// 查找方法（包括从父类继承的）
    pub fn find_method(&self, name: &str) -> Option<Gc<'gc, Function<'gc>>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }

        // 从父类查找
        if let Some(parent) = &self.parent {
            return parent.find_method(name);
        }

        None
    }

    /// 创建该类的实例
    pub fn instantiate(&self, mc: &Mutation<'gc>) -> Gc<'gc, NyarObject<'gc>> {
        let obj = NyarObject {
            class: Some(unsafe {
                Gc::from_ptr(self as *const _)
            }),
            enum_type: None,
            variant: None,
            properties: HashMap::new(),
        };

        Gc::new(mc, obj)
    }
}

impl<'gc> Trait<'gc> {
    /// 创建一个新的特征
    pub fn new(mc: &Mutation<'gc>, name: &str) -> Gc<'gc, Self> {
        Gc::new(mc, Self {
            name: Gc::new(mc, name.to_string()),
            parents: Vec::new(),
            method_signatures: HashMap::new(),
        })
    }

    /// 添加方法签名
    pub fn add_method_signature(&mut self, signature: MethodSignature) {
        self.method_signatures.insert(signature.name.clone(), signature);
    }

    /// 检查类是否实现了该特征
    pub fn is_implemented_by(&self, class: &Class<'gc>) -> bool {
        // 检查类是否直接实现了该特征
        if class.traits.iter().any(|t| t.name.as_ref() == self.name.as_ref()) {
            return true;
        }

        // 检查类是否实现了该特征的父特征
        for parent_trait in &self.parents {
            if parent_trait.is_implemented_by(class) {
                return true;
            }
        }

        // 检查父类是否实现了该特征
        if let Some(parent) = &class.parent {
            if self.is_implemented_by(parent) {
                return true;
            }
        }

        false
    }
}

impl<'gc> Enum<'gc> {
    /// 创建一个新的枚举
    pub fn new(mc: &Mutation<'gc>, name: &str) -> Gc<'gc, Self> {
        Gc::new(mc, Self {
            name: Gc::new(mc, name.to_string()),
            variants: HashMap::new(),
            methods: HashMap::new(),
        })
    }

    /// 添加变体
    pub fn add_variant(&mut self, variant: EnumVariant<'gc>) {
        self.variants.insert(variant.name.clone(), variant);
    }

    /// 添加方法
    pub fn add_method(&mut self, name: &str, method: Gc<'gc, Function<'gc>>) {
        self.methods.insert(name.to_string(), method);
    }

    /// 创建变体实例
    pub fn instantiate_variant(
        &self,
        mc: &Mutation<'gc>,
        variant_name: &str,
        args: Vec<NyarValue<'gc>>,
    ) -> Result<Gc<'gc, NyarObject<'gc>>, nyar_error::NyarError> {
        if let Some(variant) = self.variants.get(variant_name) {
            let obj = NyarObject {
                class: None,
                enum_type: Some(unsafe {
                    Gc::from_ptr(self as *const _)
                }),
                variant: Some(variant_name.to_string()),
                properties: HashMap::new(),
            };

            // 设置字段值
            let obj_ref = Gc::new(mc, obj);
            let mut obj_ref = obj_ref.write(mc);

            for (i, field_name) in variant.fields.iter().enumerate() {
                if i < args.len() {
                    obj_ref.properties.insert(field_name.clone(), args[i].clone());
                } else {
                    obj_ref.properties.insert(field_name.clone(), NyarValue::Null);
                }
            }

            Ok(obj_ref)
        } else {
            Err(nyar_error::NyarError::custom(format!(
                "Enum {} does not have variant {}",
                self.name, variant_name
            )))
        }
    }
}

impl<'gc> NyarObject<'gc> {
    /// 获取属性值
    pub fn get_property(&self, name: &str) -> Option<NyarValue<'gc>> {
        self.properties.get(name).cloned()
    }

    /// 设置属性值
    pub fn set_property(&mut self, name: &str, value: NyarValue<'gc>) {
        self.properties.insert(name.to_string(), value);
    }

    /// 调用方法
    pub fn call_method(
        &self,
        name: &str,
        args: Vec<NyarValue<'gc>>,
    ) -> Option<Gc<'gc, Function<'gc>>> {
        if let Some(class) = &self.class {
            class.find_method(name)
        } else if let Some(enum_type) = &self.enum_type {
            enum_type.methods.get(name).cloned()
        } else {
            None
        }
    }
}