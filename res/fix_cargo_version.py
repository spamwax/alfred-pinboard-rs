import toml
import sys

c = toml.load("Cargo.toml")
c['package']['version'] = sys.argv[1]
c['profile'] = {'release': {'lto': True}}

f = open("Cargo.toml", "w")
_ = toml.dump(c, f)
