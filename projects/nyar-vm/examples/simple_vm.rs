//! 简单的虚拟机使用示例

use nyar_vm::{Instruction, Value, VirtualMachine};

fn main() {
    // 创建虚拟机实例
    let mut vm = VirtualMachine::new();
    
    // 创建一个简单的指令序列
    // 1. 将常量1压入栈
    // 2. 将常量2压入栈
    // 3. 将变量"x"设置为栈顶值(2)
    // 4. 将变量"y"设置为栈顶值(1)
    // 5. 将变量"x"压入栈
    // 6. 将变量"y"压入栈
    let instructions = vec![
        Instruction::PushConstant { value: Value::BigInt(1) },
        Instruction::PushConstant { value: Value::BigInt(2) },
        Instruction::StoreVariable { name: "x".to_string() },
        Instruction::StoreVariable { name: "y".to_string() },
        Instruction::PushVariable { name: "x".to_string() },
        Instruction::PushVariable { name: "y".to_string() },
    ];
    
    // 执行指令序列
    match vm.execute(instructions) {
        Ok(result) => {
            println!("执行成功，最后的结果是: {:?}", result);
        }
        Err(err) => {
            println!("执行失败: {}", err);
        }
    }
    
    // 协程示例
    println!("\n执行协程示例:");
    coroutine_example();
}

/// 协程示例
fn coroutine_example() {
    let mut vm = VirtualMachine::new();
    
    // 创建一个协程函数的指令序列
    // 1. 创建一个函数，它会产生三个值: 1, 2, 3
    let function_body = vec![
        Instruction::PushConstant { value: Value::BigInt(1) },
        Instruction::YieldCoroutine { value_count: 1 },
        Instruction::PushConstant { value: Value::BigInt(2) },
        Instruction::YieldCoroutine { value_count: 1 },
        Instruction::PushConstant { value: Value::BigInt(3) },
        Instruction::Return,
    ];
    
    // 创建函数并转换为协程
    let create_function_instructions = vec![
        // 创建函数 (无参数)
        Instruction::CreateFunction { 
            name: Some("generator".to_string()), 
            parameter_count: 0, 
            body_size: function_body.len() 
        },
    ];
    
    // 将函数体附加到指令后面
    let mut instructions = create_function_instructions;
    instructions.extend(function_body);
    
    // 将函数转换为协程并执行
    instructions.push(Instruction::CreateCoroutine);
    
    // 执行指令序列创建协程
    let coroutine_result = vm.execute(instructions);
    
    if let Ok(coroutine) = coroutine_result {
        println!("协程创建成功");
        
        // 恢复协程三次，获取所有值
        let resume_instructions = vec![Instruction::ResumeCoroutine];
        
        for i in 1..=3 {
            // 将协程放入栈中
            let mut resume_with_coroutine = vec![Instruction::PushConstant { 
                value: Value::Coroutine(coroutine.transmute()) 
            }];
            resume_with_coroutine.extend(resume_instructions.clone());
            
            match vm.execute(resume_with_coroutine) {
                Ok(value) => {
                    println!("第{}次恢复协程，得到值: {:?}", i, value);
                }
                Err(err) => {
                    println!("恢复协程失败: {}", err);
                    break;
                }
            }
        }
    } else {
        println!("创建协程失败: {:?}", coroutine_result.err());
    }
}