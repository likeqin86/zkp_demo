mod demo_mimc;

extern crate ff;
extern crate rand;
extern crate bellman;
extern crate bls12_381;

//非常重要的引入
use ff::Field;
use bls12_381::{Scalar, Bls12};
use bellman::groth16;
use bellman::groth16::Proof;

use rand::thread_rng;

pub fn zkp_process_add(param_a : u64, param_b : u64) ->bool {

    println!("1.Applying zkp-Demo 构造阶段-------------------");
    let mut rng = thread_rng();
    // 1.1随机树产生器产生一些列的随机数，用vec保存起来
    let constants = (0..demo_mimc::MIMC_ROUNDS)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();

    // 1.构造随机值与输入参数阶段
    // 1.2为电路生成随机参数,调用Groth16算法库函数:这里会产生vk
    let demo_rand_params = {
        //生成一个随机电路：op_param 可以任意值
        let rand_circuit = demo_mimc::MiMCDemo {
            xl: None,
            xr: None,
            constants: &constants,
        };
        //内部调用 synthesize 方法
        groth16::generate_random_parameters::<Bls12, _, _>(rand_circuit, &mut rng).unwrap()
    };

    // 1.3为证明验证提供验证key,返回了椭圆曲线上的两个点组集合，以上均为构造参数阶段
    // 很关键的数据pvk：在生成证明的时候会用到，在证明验证的时候会用到。
    // 由随机组合数集合 constants->demo_rand_params->pvk 推导来保证，每一次tx的zkp随机性
    let pub_pvk = groth16::prepare_verifying_key(&demo_rand_params.vk);

    println!("2.Applying zkp-Demo 创建证明-------------------");
    // 2.生成零知识证明阶段：Groth16为上层应用，透明化了G1、G2点集合的生成规则、椭圆曲线加解密等过程；
    // 要实现电路自定义，需要完成关于电路逻辑计算的trait实现：impl<'a, S: PrimeField> Circuit<S>
    // 本实例由MiMCDemo来完成Circuit工作，下边开始完成：证明生成阶段事宜

    // 2.1 用于存放生成证明类数据
    let mut proof_vec = vec![];
    proof_vec.truncate(0);

    // 2.2 模拟随机产生一份别样的数据，在校验证明中使得不透漏原本真实信息
    // 但是要保证：构造阶段的数据常量性constants
    let mut xl = Scalar::from(param_a);
    let mut xr = Scalar::from(param_b);

    //模拟重要一步：链下计算，即交易
    xl = xl.add(&xr);
    xr = xr.sub(&xr);

    // 验证阶段需要此数据字段作为非正式数据证明,返回电路变换之后的电路逻辑值，此数值参与最后阶段的证明验证
    // 可以简单理解为逻辑加法计算,,并且在验证阶段：不会透漏参数 param_a 和 param_b
    // 操作实质：对参数进行一定规则的汇总计算保存，从头到尾仅一次性输入参数即可
    let demo_logic = demo_mimc::mimc(xl, xr, &constants);

    // 2.3 创建电路实例：MiMCDemo里边涵盖有较为复杂的电路逻辑计算
    // 类似于构造具体交易数据
    let demo_circuit = demo_mimc::MiMCDemo {
        xl: Some(xl),
        xr: Some(xr),
        constants: &constants,
    };

    // 2.4 用我们要求的数据创建一份证明文档，并将其结果返回用于下一个阶段做证明验证
    let proof_obj = groth16::create_random_proof(demo_circuit, &demo_rand_params, &mut rng).unwrap();
    // 2.5 把证明数据proof_obj保存到proof_vec容器中，并得到一份有待证明的完整数据
    proof_obj.write(&mut proof_vec).unwrap();

    println!("3.Applying zkp-Demo 验证证明-------------------");
    // 3.验证零知识证明的数据校验阶段
    // 3.1 从存放证明的对象proof_vec内读取数据到proof_read，比如类似于读取L2层的块交易数据
    let proof_history = Proof::read(&proof_vec[..]).unwrap();

    // 3.2 验证数据的正确性，相当于在L1层检验来自L2的零知识数据，，相当于解密比对数据的过程
    let b_ok= groth16::verify_proof(&pub_pvk,&proof_history,&[demo_logic]).is_ok();
    println!("     zkp-demo Verifying process result:{}", b_ok);

    return b_ok;
}

/// 题目要求：使用rust编写一个零知识证明demo：
/// 1、基于bellman或franklin-crypto框架，实现零知识证明完整过程。
/// a) 要求待证明a=1，b=2，设计电路MyCircuit实现vk、proof的证明过程。需证明a和b的求和等于3,但不泄露a和b的具体值。
/// b) 详细描述setup过程生成的文件的作用以及输出电路大小。
/// c) 要求实现过程增加注释。
/// 本次解决方法：当前Demo采用bellman来实现功能，算法原理更加直观、易于理解与代码审查。
/// 2、实现聚合电路证明完整过程。（加分项）

fn main() {

    println!("=================Applying zkp-Demo process start!=================");
    zkp_process_add(1, 2);
    println!("=================Applying zkp-Demo process end!=================");
}
