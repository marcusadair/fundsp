use super::*;
use generic_array::sequence::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// AudioComponent processes audio data sample by sample.
/// It has a static number of inputs and outputs known at compile time.
/// If not set otherwise, the sample rate is presumed the system default DEFAULT_SR.
pub trait AudioComponent: Clone
{
    type Inputs: Size;
    type Outputs: Size;

    /// Resets the input state of the component to an initial state where it has not processed any samples.
    /// In other words, resets time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs; others should return 0.0.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> f64 { 0.0 }
    // TODO: latency needs to be an option.

    // End of interface. There is no need to override the following.

    /// Number of inputs.
    #[inline] fn inputs(&self) -> usize { Self::Inputs::USIZE }

    /// Number of outputs.
    #[inline] fn outputs(&self) -> usize { Self::Outputs::USIZE }

    /// Retrieves the next mono sample from an all-zero input.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline] fn get_mono(&mut self) -> f48 {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        output[0]
    }

    /// Retrieves the next stereo sample pair (left, right) from an all-zero input.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline] fn get_stereo(&mut self) -> (f48, f48) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        (output[0], output[ if self.outputs() > 1 { 1 } else { 0 } ])
    }

    /// Filters the next mono sample.
    /// Broadcasts the input to as many channels as are needed.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline] fn filter_mono(&mut self, x: f48) -> f48 {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filters the next stereo sample pair.
    /// Broadcasts the input by wrapping to as many channels as are needed.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline] fn filter_stereo(&mut self, x: f48, y: f48) -> (f48, f48) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::generate(|i| if i & 1 == 0 { x } else { y }));
        (output[0], output[ if self.outputs() > 1 { 1 } else { 0 } ])
    }
}

/// PassComponent passes through its inputs unchanged.
#[derive(Clone)]
pub struct PassComponent<N: Size>
{
    _length: PhantomData<N>,
}

impl<N: Size> PassComponent<N>
{
    pub fn new() -> Self { PassComponent { _length: PhantomData::default() } }
}

impl<N: Size> AudioComponent for PassComponent<N>
{
    type Inputs = N;
    type Outputs = N;

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        input.clone()
    }
}

/// SinkComponent consumes its inputs.
#[derive(Clone)]
pub struct SinkComponent<N: Size>
{
    _length: PhantomData<N>,
}

impl<N: Size> SinkComponent<N>
{
    pub fn new() -> Self { SinkComponent { _length: PhantomData::default() } }
}

impl<N: Size> AudioComponent for SinkComponent<N>
{
    type Inputs = N;
    type Outputs = N;

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        Frame::default()
    }
}

/// ConstantComponent outputs a constant value.
#[derive(Clone)]
pub struct ConstantComponent<N: Size>
{
    output: Frame<N>,
}

impl<N: Size> ConstantComponent<N>
{
    pub fn new(output: Frame<N>) -> Self { ConstantComponent { output } }
}

impl<N: Size> AudioComponent for ConstantComponent<N>
{
    type Inputs = U0;
    type Outputs = N;

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        self.output.clone()
    }
}

#[derive(Clone)]
pub enum Binop { Add, Sub, Mul }

pub trait FrameBinop<S: Size>: Clone {
    fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S>;
}
#[derive(Clone)]
pub struct FrameAdd<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameAdd<S> {
    pub fn new() -> FrameAdd<S> { FrameAdd { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameAdd<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x + y }
}

#[derive(Clone)]
pub struct FrameSub<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameSub<S> {
    pub fn new() -> FrameSub<S> { FrameSub { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameSub<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x - y }
}

#[derive(Clone)]
pub struct FrameMul<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameMul<S> {
    pub fn new() -> FrameMul<S> { FrameMul { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameMul<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x * y }
}

#[derive(Clone)]
pub enum Unop { Neg }

pub trait FrameUnop<S: Size>: Clone {
    fn unop(x: &Frame<S>) -> Frame<S>;
}
#[derive(Clone)]
pub struct FrameNeg<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameNeg<S> {
    pub fn new() -> FrameNeg<S> { FrameNeg { _size: PhantomData::default() } }
}

impl<S: Size> FrameUnop<S> for FrameNeg<S> {
    #[inline] fn unop(x: &Frame<S>) -> Frame<S> { -x }
}

/// BinopComponent combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of outputs.
#[derive(Clone)]
pub struct BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    x: X,
    y: Y,
    b: B,
}

impl<X, Y, B> BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    pub fn new(x: X, y: Y, b: B) -> Self { BinopComponent { x, y, b } }
}

impl<X, Y, B> AudioComponent for BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let x = self.x.tick(input_x.into());
        let y = self.y.tick(input_y.into());
        B::binop(&x, &y)
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// UnopComponent applies an unary operator to its inputs.
#[derive(Clone)]
pub struct UnopComponent<X, U: FrameUnop<X::Outputs>> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    x: X,
    u: U,
}

impl<X, U> UnopComponent<X, U> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    pub fn new(x: X, u: U) -> Self { UnopComponent { x, u } }
}

impl<X, U> AudioComponent for UnopComponent<X, U> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        U::unop(&self.x.tick(input))
    }
}

/// PipeComponent pipes the output of X to Y.
#[derive(Clone)]
pub struct PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    x: X,
    y: Y,
}

impl<X, Y> PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    pub fn new(x: X, y: Y) -> Self { PipeComponent { x, y } }
}

impl<X, Y> AudioComponent for PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        self.y.tick(&self.x.tick(input))
    }
    fn latency(&self) -> f64 { self.x.latency() + self.y.latency() }
}

//// StackComponent stacks X and Y in parallel.
#[derive(Clone)]
pub struct StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    x: X,
    y: Y,
}

impl<X, Y> StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    pub fn new(x: X, y: Y) -> Self { StackComponent { x, y } }
}

impl<X, Y> AudioComponent for StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let output_x = self.x.tick(input_x.into());
        let output_y = self.y.tick(input_y.into());
        Frame::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// BranchComponent sends the same input to X and Y and concatenates the outputs.
#[derive(Clone)]
pub struct BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    x: X,
    y: Y,
}

impl<X, Y> BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    pub fn new(x: X, y: Y) -> Self { BranchComponent { x, y } }
}

impl<X, Y> AudioComponent for BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Inputs = X::Inputs;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        Frame::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// CascadeComponent pipes X to Y, adding more inputs from X in place of missing ones.
/// The number of inputs in X and Y must match.
#[derive(Clone)]
pub struct CascadeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    Y::Outputs: Size,
{
    x: X,
    y: Y,
}

impl<X, Y> CascadeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    Y::Outputs: Size,
{
    pub fn new(x: X, y: Y) -> Self { CascadeComponent { x, y } }
}

impl<X, Y> AudioComponent for CascadeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    Y::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output_x = self.x.tick(input);
        let input_y = Frame::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { input[i] });
        self.y.tick(&input_y)
    }
    fn latency(&self) -> f64 {
        // If all channels are piped through X, then take latency of X into account.
        if X::Outputs::USIZE >= Y::Inputs::USIZE {
            self.x.latency() + self.y.latency()
        } else {
            self.y.latency()
        }
    }
}
