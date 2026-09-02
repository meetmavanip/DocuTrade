const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying DocumentVerification to Arbitrum Sepolia...");

  const DocumentVerification = await ethers.getContractFactory("DocumentVerification");
  const contract = await DocumentVerification.deploy();

  await contract.waitForDeployment();

  const address = await contract.getAddress();
  console.log(`DocumentVerification deployed to: ${address}`);
  console.log(`\nAdd this to your .env file:`);
  console.log(`DOCUMENT_VERIFICATION_CONTRACT=${address}`);
  console.log(`\nVerify on Arbiscan:`);
  console.log(`npx hardhat verify --network arbitrumSepolia ${address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
