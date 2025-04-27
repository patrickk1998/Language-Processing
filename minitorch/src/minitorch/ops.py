from minitorch.autograd import Ten, Grad_Fn, Grad_Fn_Scal
from numpy.typing import NDArray, ArrayLike, DTypeLike
import numpy as np

"""
A Helper function to generate a list if Grad_FN parent nodes
"""

def create_parent(*inputs: Ten):
    parent = []
    for i in inputs:
        if(i.require_grad):
            parent.append(i.get_grad_fn())
        else:
            parent.append(None)
    return parent

def needs_grad(*inputs: Ten):
  for i in inputs:
    if i.require_grad:
      return True

"""
A forward function does two main things.
    1. Takes in the inputs and calculates the output value

    2. Attaches a Grad_Fn node, sets the position (the tensors needed to calculate the backward pass), and 
    the parents (the Grad_Fn of the input nodes)

In general the forward functions can have restrictions on the shape of the tensor. The function can also assume the semantics of the 
shape, for instance it can assume that tensor of shape (10,10) is an batch of 10 vectors of length 10, or it can assume it is a single 10 by 10 
matrix. 

The backwards function does one thing – it takes in the nodes attached to its Grad_Fn container as the position and as input 
a gradient. As right now there is no mask, so it calculates the gradient for all nodes even the parent nodes do not need it. It is important 
to note that regardless of the semantics of the input – if a dimension represents a batch – that the backwards gradient must have the following shape
    1. If one of the dimensions is a batch, then the shape must be the same
    2. If the tensor is singular, then the shape must be the same except for the top most dimension, which represents a batch of gradients to
    be summed over.

"""

"""
Add Tensors

Two tensors (batches) are added. Both must be the same shape and the output is
the same shape.
"""

"""
For adding vectors A + B, the jacobian matrix is just with respect to
A or B is just the identity. Hence we just return the gradient that was
passed in for each input.
"""
def F_add_grad_fn(position: list[Ten], input: ArrayLike) -> list[ArrayLike]:
  return [input, input]

def add(a: Ten, b: Ten) -> Ten:

  c = a.to_numpy() + b.to_numpy()
  c = Ten(c, leaf=False)

  if a.require_grad or b.require_grad:
    c.grad_on()
  else:
    return c

  c.grad_fn = Grad_Fn(F_add_grad_fn)
  parents = create_parent(a, b)
  c.grad_fn.set_parents(parents)
  c.grad_fn.set_position([a,b])
  return c

"""
Pool Tensor (Scalar ouput)

Given a batch of tensors, this function produces a batch of scalar outputs
that represnt the pooles sum of their lower dimensions.
"""

def F_pool_grad_fn(position: list[Ten], input: ArrayLike) -> list[ArrayLike]:
  return [np.ones(position[0].shape())]

def pool_ten(a: Ten):
  npa = a.to_numpy() 
  s = np.sum(npa.reshape(npa.shape[0], -1), axis=1)
  s = Ten(s, leaf=False)

  if a.require_grad:
    s.grad_on()
  else:
    return s

  s.grad_fn = Grad_Fn_Scal(F_pool_grad_fn)
  s.grad_fn.set_parents([a.get_grad_fn()])
  s.grad_fn.set_position([a])
  return s


"""
Matrix Multiply

Multiplies a bactch of one dimesional column tensors by a matrix. 

position[0]: Matrix that is used to multiply the vectors.

position[1]: Batch of column vectors that is multiplied by.
"""

def F_matmul_grad_fn(position: list[Ten], input: NDArray) -> list[ArrayLike]:
    grad_0 = np.zeros((input.shape[0], position[0].shape()[0], position[0].shape()[1]))
    x = position[1].to_numpy()
    for i in range(input.shape[0]):
      grad_0[i, :, :] = np.outer(input[i], x[i])
    return [grad_0, input @ position[0]]

def matmul(M: Ten, x: Ten):
    Mnp = M.to_numpy()
    xnp = x.to_numpy()
    tnp = xnp @ Mnp.transpose()
    
    t = Ten(tnp, leaf=False)

    if not needs_grad(M, x):
      return t

    t.grad_on()
    t.grad_fn = Grad_Fn(F_matmul_grad_fn)
    parents = create_parent(M, x)
    t.grad_fn.set_parents(parents)
    t.grad_fn.set_position([[M, x]])
