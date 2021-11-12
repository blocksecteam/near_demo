const nearAPI = require("near-api-js");
const BN = require("bn.js");   

// 用于wasm文件的读写
const fs = require("fs").promises;

const assert = require("assert").strict;
// 配置测试执行环境
function getConfig(env) {
  switch (env) {
    case "sandbox":
    case "local":
      return {
        networkId: "sandbox",
        nodeUrl: "http://localhost:3030",   
        masterAccount: "test.near",            
        contractAccount1: "attacker.test.near",    
        contractAccount2: "victim.test.near", 
        contractAccount3: "ft_token.test.near",       
        keyPath: "/tmp/near-sandbox/validator_key.json",
        };
  }
}




// let 用来声明局部变量。它的用法类似于var，但是所声明的变量，只在let命令所在的代码块内有效。
let config;
let masterAccount;
let masterKey;
let pubKey;
let keyStore;
let near;


async function initNear() {
  // 调用getConfig，做好测试环境的配置。参数指定测试的环境为sandbox
  config = getConfig(process.env.NEAR_ENV || "sandbox"); 

  // 配置masterAccount keyStore
  const keyFile = require(config.keyPath);
  masterKey = nearAPI.utils.KeyPair.fromString(
    keyFile.secret_key || keyFile.private_key
  );
  pubKey = masterKey.getPublicKey();
  keyStore = new nearAPI.keyStores.InMemoryKeyStore();
  keyStore.setKey(config.networkId, config.masterAccount, masterKey);
  
  near = await nearAPI.connect({
    deps: {
      keyStore,
    },
    networkId: config.networkId,
    nodeUrl: config.nodeUrl,
  });
  // 返回masterAccount用户对象实例，不同于config.masterAccount所指的字符串
  masterAccount = new nearAPI.Account(near.connection, config.masterAccount);
  console.log("Finish init NEAR");
}



async function createContractUser(
  accountPrefix,
  contractAccountId,
  contractMethods1
) {
  // masterAccount的子账户，例如alice.test.near
  let accountId = accountPrefix + "." + config.masterAccount;
  await masterAccount.createAccount(
    accountId,
    pubKey,
    new BN(10).pow(new BN(25))
  );
  keyStore.setKey(config.networkId, accountId, masterKey);   
  // 这里alice.test.near和test.near共享了masterKey
  // 返回
  const account = new nearAPI.Account(near.connection, accountId); 
  // 返回了alice.test.near的账户对象实例

  const accountUseContract = new nearAPI.Contract(
    account,
    contractAccountId,
    contractMethods1
  );
  return accountUseContract;
}



// contractAccount1: "attacker.test.near",    
// contractAccount2: "victim.test.near", 
// contractAccount3: "ft_token.test.near",       










// 这里填写用户可调用的合约函数
const contractMethods1 = {
  changeMethods: ['ft_on_transfer','malicious_call'],
  viewMethods:['view_gas']
};

const contractMethods2 = {
  changeMethods: ['withdraw'],
  viewMethods:['view_attacker_balance','view_gas']
};

const contractMethods3 = {
  changeMethods: ['ft_transfer_call'],
  viewMethods:['view_attacker_balance','view_victim_balance','view_gas']
};


async function initTest() {
  // attacker.test.near
  const contract1 = await fs.readFile("target/wasm32-unknown-unknown/release/malicious_contract.wasm");
  const _contractAccount1 = await masterAccount.createAndDeployContract(
    config.contractAccount1,
    pubKey,
    contract1,
    new BN(10).pow(new BN(25))
  );

  // victim.test.near
  const contract2 = await fs.readFile("../victim_contract/target/wasm32-unknown-unknown/release/victim_contract.wasm");
  const _contractAccount2 = await masterAccount.createAndDeployContract(
    config.contractAccount2,
    pubKey,
    contract2,
    new BN(10).pow(new BN(25))
  );


  // ft_token.test.near
  const contract3 = await fs.readFile("../ft_contract/target/wasm32-unknown-unknown/release/ft_token.wasm");
    const _contractAccount3 = await masterAccount.createAndDeployContract(
      config.contractAccount3,
      pubKey,
      contract3,
      new BN(10).pow(new BN(25))
    );



  // 创建合约1的常规用户alice和bob
  const aliceUseContract = await createContractUser(
    "alice",
    config.contractAccount1,
    contractMethods1
  );

  const bobUseContract = await createContractUser(
    "bob",
    config.contractAccount2,
    contractMethods2
  );
  
  // 创建合约2的常规用户tony
  const tonyUseContract = await createContractUser(
    "tony",
    config.contractAccount3,
    contractMethods3
  );


  console.log("Finish deploy contracts and create test accounts");
  return { aliceUseContract, bobUseContract, tonyUseContract };

}





// 测试的整体流程
async function test_reentrancy() {
  await initNear();
  const { aliceUseContract, bobUseContract, tonyUseContract } = await initTest();


  let alice_message = await aliceUseContract.malicious_call({
    args: {
      amount: 60,
    }, 
    gas: "300000000000000",
  });




  // 检查gas的分配情况

  // let malicious_call_gas = await aliceUseContract.view_gas({
  //   args: { }, 
  // });

  // console.log(malicious_call_gas);

  // let ft_transfer_call_gas = await bobUseContract.view_gas({
  //   args: { }, 
  // });

  // console.log(ft_transfer_call_gas);



  let bob_message = await bobUseContract.view_attacker_balance({
    args: {
    }, 
    gas: "300000000000000",
  });
  console.log("Victim::attacker_balance:" + bob_message);


  let tony_message = await tonyUseContract.view_attacker_balance({
    args: {
    }, 
  });
  console.log("FT_Token::attacker_balance:" + tony_message);

  tony_message = await tonyUseContract.view_victim_balance({
    args: {
    }, 
  });
  console.log("FT_Token::victim_balance:" + tony_message);

}

test_reentrancy();