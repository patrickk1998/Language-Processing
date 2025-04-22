import numpy as np
from abc import ABC, abstractmethod
from collections import deque
from numpy.typing import NDArray, ArrayLike, DTypeLike
from typing import Any, Callable

class Grad_Fn_Node_Exception(Exception):
    """Exception for Grad_Fn_Node objects"""
    def __init__(self, message, node):
        self.message = message
        self.node = node
        super().__init__(self.message)

class Grad_Fn_Node:
    pass

class Grad_Fn_Node_IN(ABC, Grad_Fn_Node):
  """
  This is an abstract class that represents intermediate nodes used for autograd that
  are built during the forward pass.

    1. `is_from_scalar(self)` returns true is the node is associated with a
    scalar tensor, and hence does not need an input gradient.

    2. __call__ must output a gradient for each of its of the associated
    forward functions, meaning the length of its output list must match
    the length of the list of parents.
  """

  def __init__(self, backwards_fn : Callable[..., list[ArrayLike]]):
    self.backwards_fn = backwards_fn
    self.position = None
    self.parents = None

  def __call__(self, input_grad: ArrayLike) -> list[ArrayLike]:
    if(self.position is None):
      raise Grad_Fn_Node_Exception("No position set for this node", self)
    if(not callable(self.backwards_fn)):
      raise Grad_Fn_Node_Exception("No backwards functions for this node", self)
    if(self.parents is None):
      raise Grad_Fn_Node_Exception("No parents on non-leaf node", self)
    output = self.backwards_fn(self.position, input_grad)
    if(len(output) != len(self.parents)):
      raise Grad_Fn_Node_Exception("Output length does not match number of parents", self)
    return output

  @abstractmethod
  def is_from_scalar(self) -> bool:
    pass

  def set_parents(self, parents):
    self.parents = parents

  def get_parents(self):
    return self.parents

  def set_position(self, position):
    self.position = position

class Grad_Fn(Grad_Fn_Node_IN):
  """
  Container for backwards gradient function for an non-leaf, non-scalar node.
  If attached to a variable z = f(x), then given a gradient dL/dz, it will
  create the gradient dL/dx. The container holds pointers to the next parents
  nodes, as well as the backwards gradient function, and gradient position.
  """

  def __init__(self, backwards_fn):
    super().__init__(backwards_fn)

  def is_from_scalar(self):
    return False

class Grad_Fn_Scal(Grad_Fn_Node_IN):
  """
  Conitainer for backwards gradient functions for non-leaf, scalar node.
  If attached to a variable s = f(x), then that variables must have shape (),
  and None can be used for the gradient in __call__. A backwards DAG graph
  traversal must start with a node of this type.
  """
  def __init__(self, backwards_fn):
    super().__init__(backwards_fn)

  def is_from_scalar(self):
    return True


class Grad_Fn_Accum(Grad_Fn_Node):
  """
  Container for a gradient accumulator, it is attacehd to a leaf node. It
  has no parents. A backwards DAG graph traversal must end with a node of this
  type
  """
  def __init__(self, tensor):
    self.tensor = tensor
    self.shape = tensor.shape()
    self.grad_accum = np.zeros(self.shape)

  def __call__(self, grad):
    if(self.tensor is None):
      raise Grad_Fn_Node_Exception("No tensor attached to accumulation node", self)
    print(f"Accumulating {grad}")
    if(grad.ndim > 1):
      sum = np.sum(grad, axis=0)
    else:
      sum = np.sum(grad.reshape(1,-1), axis=0)
    print(f"Summed up set of {sum}")
    self.grad_accum += sum

  def is_from_scalar(self):
    return False

  def set_parents(self):
    raise Grad_Fn_Node_Exception("Attempting to set parents on accumulation node", self)

  def get_parents(self):
    raise Grad_Fn_Node_Exception("Attempting to get parents from accumulation node", self)

  def grad_zero(self):
      self.grad_accum = np.zeros
    

"""
We want miniTensor to be a wrapper around a class of objects. They could be
numpy arrays, sometimes in shared memory, they can be arrays in GPU memory,
or they can be remote objects. For remote objects, we want a interface that
can name tensors (and their associated Fn_grad objects) to a computer network.
"""
class Ten:

  def __init__(self, numpy_array: NDArray, leaf=True):
    self.narray = numpy_array
    self.grad_fn : Grad_Fn_Node | None | Grad_Fn_Accum = None
    self.require_grad = False
    self.leaf = leaf

  def shape(self):
    return self.narray.shape

  def to_numpy(self):
    return self.narray

  def get_grad_fn(self) -> Grad_Fn_Node | Grad_Fn_Accum :
    if(self.grad_fn is None):
      raise Exception("No fn_grad for this tensor")
    return self.grad_fn

  def grad_on(self):
    self.require_grad = True
    if(self.leaf):
      self.grad_fn = Grad_Fn_Accum(self)

  def grad_zero(self):
    if(not self.require_grad):
      raise Exception("autograd off for this vector")
    if(self.grad_fn is Grad_Fn_Accum):
      self.grad_fn.grad_zero()

  


def add_grad_fn(grad_accum, grad, input):
  grad_accum += grad
  return grad

def sum_grad_fn(grad_accum, grad, input):
  g = np.ones(input.shape)
  grad_accum += g
  return g


"""
For adding vectors A + B, the jacobian matrix is just with respect to
A or B is just the identity. Hence we just return the gradient that was
passed in for each input.
"""
def F_add_grad_fn(position, input):
  return [input, input]

def add(a: Ten, b: Ten):

  c = a.to_numpy() + b.to_numpy()
  c = Ten(c, leaf=False)

  if a.require_grad or b.require_grad:
    c.grad_on()
  else:
    return c

  c.grad_fn = Grad_Fn(F_add_grad_fn)
  parents = []

  if(a.require_grad):
    parents.append(a.get_grad_fn())
  else:
    parents.append(None)
  if(b.require_grad):
    parents.append(b.get_grad_fn())
  else:
    parents.append(None)
  c.grad_fn.set_parents([a.get_grad_fn(), b.get_grad_fn()])
  c.grad_fn.set_position([a,b])
  return c

def F_sum_grad_fn(position, input) -> list[ArrayLike]:
  return [np.ones(position[0].shape())]

def sum_ten(a: Ten):
  s = np.sum(a.to_numpy())
  s = Ten(s, leaf=False)

  if a.require_grad:
    s.grad_on()
  else:
    return s

  s.grad_fn = Grad_Fn_Scal(F_sum_grad_fn)
  s.grad_fn.set_parents([a.get_grad_fn()])
  s.grad_fn.set_position([a])
  return s

def visit(node : Grad_Fn_Node_IN, gradient):
  if(isinstance(node, Grad_Fn_Accum)):
    node(gradient)
    return
  output = node(gradient)
  parents = node.get_parents()
  if(parents is not None):
    for i in range(len(output)):
      if(parents[i] is not None):
        visit(parents[i], output[i])
  return

def backwards(s : Ten):
  fn = s.get_grad_fn()
  if(not isinstance(fn, Grad_Fn_Scal)):
    raise Exception("S is not a scalar")
  else:
    visit(fn, None)
