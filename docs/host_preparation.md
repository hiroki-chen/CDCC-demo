# Host Machine Requirements

Currently this demo only runs on a TDX-enabled CPU and an OS with TDX features enabled. See [this](https://cc-enabling.trustedservices.intel.com/intel-tdx-enabling-guide/04/hardware_setup/#enable-intel-tdx-in-bios) for more information.

## Verify if TDX is enabled

You can check this via `sudo dmesg | grep -i tdx`. If the ring buffer has been overflown, consider check `sudo cat /var/log/syslog | grep -i tdx`:

```txt
tdx: TDX module initialized.
```

and check MSRs:

```sh
$ sudo rdmsr -f 1:! 0x982
1
$ sudo rdmsr 0xa0
0
$ sudo rdmsr -f 11:!1 0x1401
1
```

## Set up Quote Generation Service for Remote Attestation

To generate a TD quote, the guest will first need to fetch the report from the hardware and relay it to the TD quoting enclave that verifies and signs the report on the same platform. To serve the incoming report, a QGC that runs in the *host* must be alive. Install it if you have not done it.


```sh
echo 'deb [signed-by=/etc/apt/keyrings/intel-sgx-keyring.asc arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu noble main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
wget https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
sudo mkdir -p /etc/apt/keyrings
cat intel-sgx-deb.key | sudo tee /etc/apt/keyrings/intel-sgx-keyring.asc > /dev/null
sudo apt-get update
sudo apt install -y \
    tdx-qgs \
    libsgx-dcap-default-qpl \
    libsgx-dcap-ql
```

You also need a collateral caching service called Provisioning Certificate Caching service (PCCS). See [here](https://cc-enabling.trustedservices.intel.com/intel-tdx-enabling-guide/02/infrastructure_setup/).

## How to fetch the collateral from Intel

You need to register the platform and get `pckid_retireval.csv` via the tool `PCKIDRetrievalTool`. This allows you to get the so-called "enc_ppid".

Using that you can fetch the collateral using your subscription key to the PCK service from Intel.


Topology:

```txt
                                    ┌─────────────────────────────┐
   ┌────────────────────┐           │   ┌─────────────────────┐   │
   │                    │           │   │                     │   │
   │                    │           │   │                     │   │
   │                    │    gRPC   │   │                     │   │
   │          ┌─────────┼───────────┼───┤         TDX         │   │
   │          │         │           │   │                     │   │
   │          │         │           │   │                     │   │
   │Host      │         │           │   │                     │   │
   └──────────┼─────────┘           │   └─────────────────────┘   │
              │                     │                             │
              │ gRPC                │             QEMU            │
              │                     └─────────────────────────────┘
   ┌──────────┴─────────┐                                          
   │                    │                                          
   │                    │                                          
   │                    │                                          
   │       Client       │                                          
   │                    │                                          
   │                    │                                          
   │                    │                                          
   └────────────────────┘                                                                                                                                                                                
```
