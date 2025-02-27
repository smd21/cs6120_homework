import sys, json, functools
import networkx as nx

class Block:
  def __init__(self, label):
    self.label = label
    self.instructions = []
  def __init__(self, insns, label):
    self.label = label
    self.instructions = insns
  def __str__(self):
    res = ""
    for ins in self.instructions:
      res += str(ins) + "\n"
    return res
  def __hash__(self):
    return hash(self.label)
  
  def __eq__(self, other):
    if(isinstance(other, self.__class__)):
      return self.label == other.label
    else:
      return False
  def __ne__(self, other):
    return not(self==other)

def get_cfg(func):
  # make blocks
  label_blocks = dict()
  blocks : list[Block] = []
  current_block = []
  curr_label = 0

  for insn in func["instrs"]:
    if("op" in insn):
      # terminator insn
      if(insn["op"]=="br" or insn["op"]=="jmp"):
        current_block.append(insn)
        new_block = Block(current_block, curr_label)
        blocks.append(new_block)
        current_block=[]
        curr_label+=1
      else:
        current_block.append(insn)
    # label - make new block. label will always be first insn in block
    elif("label" in insn):
      if(len(current_block)>0):
        new_block = Block(current_block, curr_label)
        blocks.append(new_block)
        current_block=[]
        curr_label+=1
      
      current_block.append(insn)
    
    # this shouldnt run but here just in case
    else:
      current_block.append(insn)
  
  #push last block in fn
  new_block = Block(current_block, curr_label)
  blocks.append(new_block)

  # collect labels
  for block in blocks:
    if("label" in block.instructions[0]):
      label_blocks.update({block.instructions[0]["label"]: block})

  # iterate over blocks and make cfg - use diGraph class in networkx
  # holy hell networkx makes this so much easier tf
  cfg = nx.DiGraph()

  for i in range(len(blocks)):
    block = blocks[i]
    last_insn = block.instructions[len(block.instructions)-1]
    if("op" in last_insn): # ngl this check is probably meaningless
      if(last_insn["op"]=="br"): 
        for label in last_insn["labels"]:
          to_jump :Block = label_blocks.get(label)
          cfg.add_edge(block, to_jump)

      elif(last_insn["op"]=="jmp"):
        label = last_insn["labels"][0]
        to_jump :Block = label_blocks.get(label)
        cfg.add_edge(block, to_jump)
      else:
        if(i<len(blocks)-1):
          cfg.add_edge(block, blocks[i+1])

  return (blocks, cfg)

def dominators(blocks:list[Block], cfg:nx.DiGraph):
  # // worklist algorithm pseudocode:
  # // in[entry] = init 
  # //out[*] = init
  # // worklist = all blocks
  # // while worklist is not empty:
  # // b = pick any block from worklist
  # in[b] = merge(out[p] for every predecessor p of b)
  # // out[b] = transfer(b, in[b])
  # // if out[b] changed:
  # // worklist += successors of b
  # merge fn is intersection of all predecessors

  worklist = blocks[1:] # excluding entry
  # inputs and outputs are mappings of a block to a set of blocks
  inputs = dict() 
  outputs: dict[Block, set] = dict()
  for b in blocks:
    inputs.update({b: set()})
    outputs.update({b: set(blocks)})
  
  outputs.update({blocks[0]: [blocks[0]]}) # update entry to only be dominated by itself

  while(len(worklist)>0):
    b = worklist.pop()
    preds = cfg.predecessors(b) # THERE'S A METHOD WAHOO
    
    # in[b] = merge(out[p] for every predecessor p of b)
    folded = functools.reduce(lambda acc, x: acc.intersection(outputs.get(x)), preds, set(blocks)) #initial has to be set of all blocks bc intersection
    inputs.update({b: folded})

    # out[b] = transfer(b, in[b])
    old_out = outputs.get(b)
    new_out = folded.copy()
    new_out.add(b)
    outputs.update({b: new_out})

    if(len(old_out.difference(new_out))>0 or len(new_out.difference(old_out))>0):
      worklist.extend(cfg.successors(b)) 
  return outputs

def merge_fn(acc:set, p:Block, outputs:dict):
  outs = outputs.get(p)
  return acc.intersection(outs)

# begin methods to check stuff
def check_doms(cfg : nx.DiGraph, dominators:dict[Block, set], entry_node: Block):
  #   if i really cant figure this out use all_simple_paths
  for (block, doms) in dominators.items():
    # set of nodes from entry to block
    nodes_btwn = get_nodes_between(entry_node, block, cfg)

    # get all paths from entry to block
    paths = []
    get_all_paths(entry_node, block, cfg, set(), nodes_btwn, [], paths)
    acc = set(nodes_btwn) # initialize to all valid nodes
    for p in paths:
      acc = acc.intersection(p)
    
    # check exactly the same
    if(not len(acc.symmetric_difference(doms))==0):
      print(f"block: {block}")
      print("acc:")
      for a in acc:
        print(f"{a}")
      print("doms:")
      for d in doms:
        print(f"{d}")
      print("fail")
      


def get_nodes_between(start, end, graph: nx.DiGraph):
  nodes = set()
  # iterate backwards starting from end
  visited = set()
  not_visited = [end]
  while(len(not_visited) > 0):
    node = not_visited.pop()
    visited.add(node)
    preds = graph.predecessors(node) # if it isn't in the predecessor train than it isn't between
    for p in preds:
      if(not(p in visited)):
        not_visited.append(p)
    nodes.add(node)
  
  return nodes


def get_all_paths(current, end, graph:nx.DiGraph, visited_nodes:set, valid_nodes: set, curr_path: list, paths: list[set]):
  if(len(visited_nodes.symmetric_difference(valid_nodes))==0):
    return
  
  curr_path.append(current)

  # if we are at the desired node end path
  if(current==end):
    paths.append(set(curr_path))
  else:
    for s in graph.successors(current):
      if(s in valid_nodes): # add check for back edges - don't look if this is a loop
        # check current -> s is not already in path
        if(not(seq_in_path([current, s], curr_path))):
          get_all_paths(s, end, graph, visited_nodes, valid_nodes, curr_path, paths)
  visited_nodes.add(current)
  # adjust path back by one
  curr_path.pop()

def seq_in_path(seq:list, path:list):
  for i in range(len(path) - len(seq)):
    slice = path[i:i+len(seq)]
    if(slice==seq):
      return True
  return False

if __name__=="__main__":

  with open(sys.argv[1]) as bril_f:
    program = json.load(bril_f)
    for func in program["functions"]:
      blocks, cfg = get_cfg(func)
      doms:dict[Block, set] = dominators(blocks, cfg)
      check_doms(cfg, doms, blocks[0])

      # check dominators




  # test for get paths function
  #       1
  #       |  \
  #       2   |
  #      / \  |
  #     4    3
  #          | \
  #          5  |
  #          |  /
  #           6

  # test_graph = nx.DiGraph() (add edges)
  # paths=[]
  # get_all_paths(1, 4, test_graph, set(), set([1, 2, 3, 4, 5, 6]), [], paths)
  # print(paths)