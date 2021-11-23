const nearAPI = require("near-api-js");
const BN = require("bn.js");

// 用于wasm文件的读写
const fs = require("fs").promises;

const assert = require("assert").strict;
const { userInfo } = require("os");
// 配置测试执行环境
function getConfig(env) {
  switch (env) {
    case "sandbox":
    case "local":
      return {
        networkId: "sandbox",
        nodeUrl: "http://localhost:3030",
        masterAccount: "test.near",
        contractAccount1: "blocked_contract.test.near",
        contractAccount2: "ft_token.test.near",
        keyPath: "/tmp/near-sandbox/validator_key.json",
      };
  }
}

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




// 这里填写用户可调用的合约函数
const contractMethods1 = {
  changeMethods: ['register_account', 'distribute_token'],
};

const contractMethods2 = {
  changeMethods: ['register_account', 'ft_transfer', 'view_accounts'],
};


async function initTest() {
  // blocked_contract init
  const contract1 = await fs.readFile("target/wasm32-unknown-unknown/release/dos_contract.wasm");
  const _contractAccount1 = await masterAccount.createAndDeployContract(
    config.contractAccount1,
    pubKey,
    contract1,
    new BN(10).pow(new BN(25))
  );

  const user0UseContract1 = await createContractUser(
    "user0",
    config.contractAccount1,
    contractMethods1
  );

  const user1UseContract1 = await createContractUser(
    "user1",
    config.contractAccount1,
    contractMethods1
  );

  const user2UseContract1 = await createContractUser(
    "user2",
    config.contractAccount1,
    contractMethods1
  );

  const user3UseContract1 = await createContractUser(
    "user3",
    config.contractAccount1,
    contractMethods1
  );

  const user4UseContract1 = await createContractUser(
    "user4",
    config.contractAccount1,
    contractMethods1
  );

  const user5UseContract1 = await createContractUser(
    "user5",
    config.contractAccount1,
    contractMethods1
  );

  // FT_token init
  const contract2 = await fs.readFile("../ft_token_demo/target/wasm32-unknown-unknown/release/ft_demo.wasm");
  const _contractAccount2 = await masterAccount.createAndDeployContract(
    config.contractAccount2,
    pubKey,
    contract2,
    new BN(10).pow(new BN(25))
  );

  const TokenOwnerUseContract2 = await createContractUser(
    "ft_token_owner",
    config.contractAccount2,
    contractMethods2
  );


  console.log("Finish deploy contracts and create test accounts");
  return {
    TokenOwnerUseContract2,
    user0UseContract1,
    user1UseContract1,
    user2UseContract1,
    user3UseContract1,
    user4UseContract1,
    user5UseContract1
  };

}


// 测试的整体流程
async function test_dos() {
  await initNear();
  const {
    TokenOwnerUseContract2,
    user0UseContract1,
    user1UseContract1,
    user2UseContract1,
    user3UseContract1,
    user4UseContract1,
    user5UseContract1
  } = await initTest();



  // Token init
  let ft_token_message = null;
  ft_token_message = await TokenOwnerUseContract2.register_account({
    args: {
      account_id: "ft_token_owner.test.near"
    },
    gas: "300000000000000",
  });


  // register blocked_contract.test.near account in ft_token.test.near
  ft_token_message = await TokenOwnerUseContract2.register_account({
    args: {
      account_id: "blocked_contract.test.near"
    },
    gas: "300000000000000",
  });

  ft_token_message = await TokenOwnerUseContract2.view_accounts({
    args: {
      account_id: "blocked_contract.test.near"
    },
    gas: "300000000000000",
  });

  console.log("blocked_contract.test.near balance:" + ft_token_message)


  // token_owner will tansfer 10000 tokens to blocked_contract.test.near
  ft_token_message = await TokenOwnerUseContract2.ft_transfer({
    args: {
      receiver_id: "blocked_contract.test.near",
      amount: 10000
    },
    gas: "300000000000000",
  });


  ft_token_message = await TokenOwnerUseContract2.view_accounts({
    args: {
      account_id: "blocked_contract.test.near"
    },
    gas: "300000000000000",
  });

  console.log("blocked_contract.test.near balance:" + ft_token_message)
  console.log('now blocked_contract.test.near has 10000 Tokens.')






  // register other token users on ft_token.test.near FT token contract
  // "user3.test.near" will not be registered
  const arr = [
    "user1.test.near",
    "user2.test.near",
    "user3.test.near",
    "user4.test.near",
    "user5.test.near",
  ];
  for (let i = 0; i < arr.length; i++) {
    ft_token_message = await TokenOwnerUseContract2.register_account({
      args: {
        account_id: arr[i]
      },
      gas: "300000000000000",
    });

    ft_token_message = await TokenOwnerUseContract2.view_accounts({
      args: {
        account_id: arr[i]
      },
      gas: "300000000000000",
    });

    console.log(arr[i] + " registered with balance:" + ft_token_message)
  }



  //
  user_message = await user1UseContract1.register_account({
    args: {},
    gas: "300000000000000",
  });

  user_message = await user2UseContract1.register_account({
    args: {},
    gas: "300000000000000",
  });

  user_message = await user3UseContract1.register_account({
    args: {},
    gas: "300000000000000",
  });

  user_message = await user4UseContract1.register_account({
    args: {},
    gas: "300000000000000",
  });

  user_message = await user5UseContract1.register_account({
    args: {},
    gas: "300000000000000",
  });



  user_message = await user0UseContract1.distribute_token({
    args: {
      amount: 100
    },
    gas: "30000000000000",
  });


  for (let i = 1; i < arr.length; i++) {
    ft_token_message = await TokenOwnerUseContract2.view_accounts({
      args: {
        account_id: arr[i]
      },
      gas: "300000000000000",
    });

    console.log(arr[i] + " now with balance:" + ft_token_message)
  }


}

test_dos();
