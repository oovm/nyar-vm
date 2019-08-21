//! 虚拟机测试模块

use nyar_vm::{Instruction, Value, VirtualMachine, VmError};

#[test]
fn test_basic_operations() {
    let mut vm = VirtualMachine::new();
    
    // 测试基本的栈操作和变量存储
    let instructions = vec![
        Instruction::PushConstant { value: Value::BigInt(42) },
        Instruction::StoreVariable { name: "answer".to_string() },
        Instruction::PushVariable { name: "answer".to_string() },
    ];
    
    let result = vm.execute(instructions).unwrap();
    if let Value::BigInt(value) = result.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 42);
    } else {
        panic!("Expected BigInt, got {:?}", result);
    }
}

#[test]
fn test_conditional_jumps() {
    let mut vm = VirtualMachine::new();
    
    // 测试条件跳转
    // 如果条件为真，不跳转，结果为1
    // 如果条件为假，跳转，结果为2
    let instructions = vec![
        // 测试条件为真的情况
        Instruction::PushConstant { value: Value::Boolean(true) },
        Instruction::JumpIfFalse { offset: 2 },
        Instruction::PushConstant { value: Value::BigInt(1) },
        Instruction::Jump { offset: 2 },
        Instruction::PushConstant { value: Value::BigInt(2) },
        
        // 测试条件为假的情况
        Instruction::PushConstant { value: Value::Boolean(false) },
        Instruction::JumpIfFalse { offset: 2 },
        Instruction::PushConstant { value: Value::BigInt(3) },
        Instruction::Jump { offset: 2 },
        Instruction::PushConstant { value: Value::BigInt(4) },
    ];
    
    let result = vm.execute(instructions).unwrap();
    if let Value::BigInt(value) = result.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 4);
    } else {
        panic!("Expected BigInt, got {:?}", result);
    }
}

#[test]
fn test_function_calls() {
    let mut vm = VirtualMachine::new();
    
    // 创建一个简单的函数，接受一个参数并返回参数+1
    let function_body = vec![
        Instruction::PushVariable { name: "x".to_string() },
        Instruction::PushConstant { value: Value::BigInt(1) },
        // 这里应该有加法指令，但我们还没有实现，所以直接返回1
        Instruction::PushConstant { value: Value::BigInt(1) },
        Instruction::Return,
    ];
    
    let mut instructions = vec![
        // 创建函数
        Instruction::PushConstant { value: Value::String("x".to_string().into()) },
        Instruction::CreateFunction { 
            name: Some("increment".to_string()), 
            parameter_count: 1, 
            body_size: function_body.len() 
        },
    ];
    
    // 添加函数体
    instructions.extend(function_body);
    
    // 存储函数到变量
    instructions.push(Instruction::StoreVariable { name: "increment".to_string() });
    
    // 调用函数
    instructions.extend(vec![
        Instruction::PushVariable { name: "increment".to_string() },
        Instruction::PushConstant { value: Value::BigInt(5) },
        Instruction::Call { argument_count: 1 },
    ]);
    
    let result = vm.execute(instructions).unwrap();
    if let Value::BigInt(value) = result.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 1); // 由于没有实现加法，所以结果是1而不是6
    } else {
        panic!("Expected BigInt, got {:?}", result);
    }
}

#[test]
fn test_coroutines() {
    let mut vm = VirtualMachine::new();
    
    // 创建一个协程函数，产生三个值
    let function_body = vec![
        Instruction::PushConstant { value: Value::BigInt(1) },
        Instruction::YieldCoroutine { value_count: 1 },
        Instruction::PushConstant { value: Value::BigInt(2) },
        Instruction::YieldCoroutine { value_count: 1 },
        Instruction::PushConstant { value: Value::BigInt(3) },
        Instruction::Return,
    ];
    
    let mut instructions = vec![
        // 创建函数
        Instruction::CreateFunction { 
            name: Some("generator".to_string()), 
            parameter_count: 0, 
            body_size: function_body.len() 
        },
    ];
    
    // 添加函数体
    instructions.extend(function_body);
    
    // 创建协程
    instructions.push(Instruction::CreateCoroutine);
    
    // 执行创建协程的指令
    let coroutine = vm.execute(instructions).unwrap();
    
    // 第一次恢复协程
    let resume_instructions = vec![
        Instruction::PushConstant { value: Value::Coroutine(coroutine.transmute()) },
        Instruction::ResumeCoroutine,
    ];
    
    let result1 = vm.execute(resume_instructions.clone()).unwrap();
    if let Value::BigInt(value) = result1.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 1);
    } else {
        panic!("Expected BigInt, got {:?}", result1);
    }
    
    // 第二次恢复协程
    let result2 = vm.execute(resume_instructions.clone()).unwrap();
    if let Value::BigInt(value) = result2.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 2);
    } else {
        panic!("Expected BigInt, got {:?}", result2);
    }
    
    // 第三次恢复协程
    let result3 = vm.execute(resume_instructions.clone()).unwrap();
    if let Value::BigInt(value) = result3.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 3);
    } else {
        panic!("Expected BigInt, got {:?}", result3);
    }
}

#[test]
fn test_algebraic_effects() {
    let mut vm = VirtualMachine::new();
    
    // 创建一个效应处理器函数
    let handler_body = vec![
        // 获取效应参数
        Instruction::PushVariable { name: "x".to_string() },
        // 将参数值加倍
        Instruction::PushConstant { value: Value::BigInt(2) },
        // 这里应该有乘法指令，但我们还没有实现，所以直接返回42
        Instruction::PushConstant { value: Value::BigInt(42) },
        // 恢复效应，返回计算结果
        Instruction::ResumeEffect { value_count: 1 },
    ];
    
    // 创建一个使用效应的函数
    let function_body = vec![
        // 触发名为"double"的效应，传递参数21
        Instruction::PushConstant { value: Value::BigInt(21) },
        Instruction::RaiseEffect { name: "double".to_string(), argument_count: 1 },
        // 效应处理后继续执行，返回效应处理的结果
        Instruction::Return,
    ];
    
    let mut instructions = vec![
        // 创建效应处理器函数
        Instruction::PushConstant { value: Value::String("x".to_string().into()) },
        Instruction::CreateFunction { 
            name: Some("double_handler".to_string()), 
            parameter_count: 1, 
            body_size: handler_body.len() 
        },
    ];
    
    // 添加处理器函数体
    instructions.extend(handler_body);
    
    // 注册效应处理器
    instructions.push(Instruction::HandleEffect { name: "double".to_string() });
    
    // 创建使用效应的函数
    instructions.extend(vec![
        Instruction::CreateFunction { 
            name: Some("use_effect".to_string()), 
            parameter_count: 0, 
            body_size: function_body.len() 
        },
    ]);
    
    // 添加函数体
    instructions.extend(function_body);
    
    // 调用函数
    instructions.extend(vec![
        Instruction::Call { argument_count: 0 },
    ]);
    
    let result = vm.execute(instructions).unwrap();
    if let Value::BigInt(value) = result.deref(&vm.heap).unwrap() {
        assert_eq!(*value, 42);
    } else {
        panic!("Expected BigInt, got {:?}", result);
    }
}

#[test]
fn test_error_handling() {
    let mut vm = VirtualMachine::new();
    
    // 测试未定义变量错误
    let instructions = vec![
        Instruction::PushVariable { name: "undefined".to_string() },
    ];
    
    let result = vm.execute(instructions);
    assert!(result.is_err());
    if let Err(VmError::UndefinedVariable(name)) = result {
        assert_eq!(name, "undefined");
    } else {
        panic!("Expected UndefinedVariable error, got {:?}", result);
    }
    
    // 测试栈下溢错误
    let instructions = vec![
        Instruction::StoreVariable { name: "x".to_string() },
    ];
    
    let result = vm.execute(instructions);
    assert!(result.is_err());
    if let Err(VmError::StackUnderflow) = result {
        // 正确的错误类型
    } else {
        panic!("Expected StackUnderflow error, got {:?}", result);
    }
}