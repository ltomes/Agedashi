#!/usr/bin/env python3
"""
This is the expected Python code that Agedashi would generate
from the sample-graph.dot file.

To test this, you would need:
1. Python 3.9+
2. pip install diagrams
3. graphviz installed on your system

Then run: python3 expected-output.py
"""

from diagrams import Diagram
from diagrams.aws.compute import EC2
from diagrams.aws.database import RDS
from diagrams.aws.network import ELB
from diagrams.aws.network import VPC
from diagrams.aws.network import PublicSubnet
from diagrams.aws.security import SecurityGroup
from diagrams.aws.storage import S3

with Diagram("infrastructure", show=False, direction="TB", outformat="png"):
    node_0 = RDS("aws_db_instance.postgres")
    node_1 = PublicSubnet("aws_db_subnet_group.main")
    node_2 = EC2("aws_instance.web")
    node_3 = ELB("aws_lb.main")
    node_4 = S3("aws_s3_bucket.static")
    node_5 = SecurityGroup("aws_security_group.web")
    node_6 = PublicSubnet("aws_subnet.public")
    node_7 = VPC("aws_vpc.main")

    node_0 >> node_1
    node_0 >> node_5
    node_1 >> node_6
    node_2 >> node_5
    node_2 >> node_6
    node_3 >> node_5
    node_3 >> node_6
    node_5 >> node_7
    node_6 >> node_7
