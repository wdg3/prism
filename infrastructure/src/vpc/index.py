from constructs import Construct
from cdktf_cdktf_provider_aws.vpc import Vpc
from cdktf_cdktf_provider_aws.subnet import Subnet

class VPC(Construct):
    vpc: Vpc
    subnets: list[Subnet]

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str):
        super().__init__(scope, name + suffix + "-vpc")
        self.vpc = Vpc(scope, name + suffix + "-vpc-construct-", cidr_block="207.13.0.0/16")
        self.subnets = []
        for (az, cidr_block) in [("a", "207.13.0.0/24"), ("b", "207.13.1.0/24"), ("c", "207.13.2.0/24")]:
            # wtf?
            if region == "ap-northeast-1" and az == "b":
                az = "d"
            elif region == "us-west-1" and az == "c":
                continue
            availability_zone = region + az
            self.subnets.append(VPCSubnet(scope, name, suffix, region, environment, availability_zone, self.vpc.id, cidr_block))

class VPCSubnet(Construct):
    subnet: Subnet

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str, az: str, vpc_id: str, cidr_block: str):
        super().__init__(scope, name + suffix + "-subnet-" + az)
        self.subnet = Subnet(scope, name + suffix + "-subnet-construct-" + az, vpc_id=vpc_id, availability_zone=az, cidr_block=cidr_block)