import toml
import sys

c = toml.load("Cargo.toml")
c['package']['version'] =  sys.argv[1]


f = open("Cargo.toml", "w")
_ = toml.dump(c, f)
