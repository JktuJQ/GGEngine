//! `mathcore::matrices` submodule implements NxM matrices which can be used to apply transformations
//! on vectors.
//!

use crate::mathcore::{
    floats::{equal, FloatOperations},
    vectors::Vector2,
    Sign,
};
use serde::{Deserialize, Serialize};
use serde_big_array::Array;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// [`Matrix`] struct implements linear algebra functions with matrices.
///
/// It also implements various matrix operations with second operand being either matrix or number.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Matrix<const ROWS: usize, const COLUMNS: usize> {
    /// Underlying array.
    ///
    arr: Array<Array<f32, COLUMNS>, ROWS>,
}
impl<const ROWS: usize, const COLUMNS: usize> Matrix<ROWS, COLUMNS> {
    /// Returns count of matrix rows.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 4> = Matrix::zero();
    /// assert_eq!(matrix.rows(), 3);
    /// ```
    ///
    pub fn rows(&self) -> usize {
        ROWS
    }
    /// Returns count of matrix columns.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 4> = Matrix::zero();
    /// assert_eq!(matrix.columns(), 4);
    /// ```
    ///
    pub fn columns(&self) -> usize {
        COLUMNS
    }
    /// Returns matrix size as a tuple.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 4> = Matrix::zero();
    /// assert_eq!(matrix.size(), (3, 4));
    /// ```
    ///
    pub fn size(&self) -> (usize, usize) {
        (ROWS, COLUMNS)
    }
    /// Returns matrix as an array.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 3> = Matrix::zero();
    /// assert_eq!(matrix.as_array(), [[0.0; 3]; 3]);
    /// ```
    ///
    pub fn as_array(&self) -> [[f32; COLUMNS]; ROWS] {
        let mut arr: [[f32; COLUMNS]; ROWS] = [[0.0; COLUMNS]; ROWS];
        for (r, item) in self.arr.iter().enumerate().take(ROWS) {
            arr[r] = item.0;
        }
        arr
    }

    /// Initializes matrix with zeroes.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 4> = Matrix::zero();
    /// assert_eq!(matrix.as_array(), [[0.0; 4]; 3]);
    /// ```
    ///
    pub fn zero() -> Self {
        Self {
            arr: Array([Array([0.0; COLUMNS]); ROWS]),
        }
    }

    /// Initializes matrix with ones.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 4> = Matrix::one();
    /// assert_eq!(matrix.as_array(), [[1.0; 4]; 3]);
    /// ```
    ///
    pub fn one() -> Self {
        Self {
            arr: Array([Array([1.0; COLUMNS]); ROWS]),
        }
    }

    /// Applies function to every matrix element and returns changed matrix.
    ///
    /// Allows to perform custom operations on each matrix element.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let mut matrix: Matrix<1, 4> = Matrix::from([[1.0, 2.0, 3.0, 4.0]]);
    /// matrix = matrix.map(|x| x + 1.0);
    /// assert_eq!(matrix.as_array(), [[2.0, 3.0, 4.0, 5.0]]);
    /// ```
    ///
    pub fn map(self, f: impl Fn(f32) -> f32) -> Matrix<ROWS, COLUMNS> {
        let mut matrix: Matrix<ROWS, COLUMNS> = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                matrix[r][c] = f(self[r][c]);
            }
        }
        matrix
    }
    /// Combines matrices by applying function on their elements.
    ///
    /// Allows performing operations with 2 matrices.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let m1: Matrix<1, 4> = Matrix::from([[1.0, 2.0, 2.0, 1.0]]);
    /// let m2: Matrix<1, 4> = Matrix::from([[2.0, 1.0, 1.0, 2.0]]);
    /// assert_eq!(m1.combine(m2, |a, b| a + b).as_array(), [[3.0; 4]]);
    /// ```
    ///
    pub fn combine(
        self,
        other: Matrix<ROWS, COLUMNS>,
        f: impl Fn(f32, f32) -> f32,
    ) -> Matrix<ROWS, COLUMNS> {
        let mut matrix: Matrix<ROWS, COLUMNS> = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                matrix[r][c] = f(self[r][c], other[r][c]);
            }
        }
        matrix
    }

    /// Returns transpose of initial matrix.
    ///
    /// Interchanges its rows into columns (flips matrix over its diagonal).
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let mut matrix: Matrix<3, 3> = Matrix::from([
    ///     [1.0, 2.0, 3.0],
    ///     [1.0, 2.0, 3.0],
    ///     [1.0, 2.0, 3.0]
    /// ]);
    /// assert_eq!(
    ///     matrix.transpose().as_array(),
    ///     [
    ///         [1.0; 3],
    ///         [2.0; 3],
    ///         [3.0; 3]
    ///     ]
    /// );
    /// ```
    ///
    pub fn transpose(&self) -> Matrix<COLUMNS, ROWS> {
        let mut matrix: Matrix<COLUMNS, ROWS> = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                matrix[c][r] = self[r][c];
            }
        }
        matrix
    }

    /// `internal_rref` operates with `Vec<Vec<f32>>`.
    ///
    fn internal_rref(vec: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        let mut matrix: Vec<Vec<f32>> = vec;
        if matrix.is_empty() {
            return matrix;
        }
        let (rows, columns): (usize, usize) = (matrix.len(), matrix[0].len());

        if matrix[0][0] == 0.0 {
            let mut row_i: usize = 0;
            for (r, row) in matrix.iter().enumerate().take(rows) {
                if row[0] > 0.0 {
                    row_i = r;
                    break;
                }
            }
            for c in 0..columns {
                (matrix[row_i][c], matrix[0][c]) = (matrix[0][c], matrix[row_i][c]);
            }
        }
        let mut lead: usize = 0;
        while lead < rows {
            for r in 0..rows {
                let div: f32 = matrix[lead][lead];
                let mult: f32 = matrix[r][lead] / div;
                if r == lead {
                    for c in 0..columns {
                        matrix[lead][c] /= div;
                    }
                } else {
                    for c in 0..columns {
                        matrix[r][c] -= matrix[lead][c] * mult;
                    }
                }
            }
            lead += 1;
        }
        matrix
    }

    /// Returns reduced row echelon form of initial matrix.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let matrix: Matrix<3, 4> = Matrix::from([
    ///     [5.0, -6.0, -7.0, 7.0],
    ///     [3.0, -2.0, 5.0, -17.0],
    ///     [2.0, 4.0, -3.0, 29.0]
    /// ]);
    /// assert_eq!(
    ///     matrix.rref().correct(0).as_array(),
    ///     [
    ///         [1.0, 0.0, 0.0, 2.0],
    ///         [0.0, 1.0, 0.0, 4.0],
    ///         [0.0, 0.0, 1.0, -3.0]
    ///     ],
    /// );
    /// ```
    ///
    pub fn rref(&self) -> Matrix<ROWS, COLUMNS> {
        let mut matrix: Matrix<ROWS, COLUMNS> = *self;
        let mut m: Vec<Vec<f32>> = vec![vec![0.0; COLUMNS]; ROWS];
        for (r, row) in m.iter_mut().enumerate().take(ROWS) {
            row[..COLUMNS].copy_from_slice(&matrix[r][..COLUMNS]);
        }
        m = Self::internal_rref(m);
        for (r, row) in m.iter().enumerate().take(ROWS) {
            matrix[r][..COLUMNS].copy_from_slice(&row[..COLUMNS]);
        }
        matrix
    }

    /// Performs dot product operation on two matrices.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let m1: Matrix<1, 3> = Matrix::from([[1.0, 2.0, 3.0]]);
    /// let m2: Matrix<3, 1> = Matrix::from([[1.0], [2.0], [3.0]]);
    /// assert_eq!(m1.dot_product(m2).as_array(), [[14.0]]);
    /// ```
    ///
    pub fn dot_product<const RHS_COLUMNS: usize>(
        self,
        other: Matrix<COLUMNS, RHS_COLUMNS>,
    ) -> Matrix<ROWS, RHS_COLUMNS> {
        let mut matrix: Matrix<ROWS, RHS_COLUMNS> = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..RHS_COLUMNS {
                let mut res: f32 = 0.0;
                for k in 0..COLUMNS {
                    res += self[r][k] * other[k][c];
                }
                matrix[r][c] = res;
            }
        }
        matrix
    }
}
impl<const N: usize> Matrix<N, N> {
    /// Returns tuple of echelon form of initial matrix and determinant sign.
    ///
    fn internal_echelon_form(&self) -> (Matrix<N, N>, Sign) {
        if N == 0 {
            return (*self, Sign::Zero);
        }
        let mut matrix: Matrix<N, N> = *self;
        let mut sign: Sign = Sign::Positive;
        let size: usize = N;
        for r in 0..(size - 1) {
            for c in ((r + 1)..size).rev() {
                if matrix[c][r] == 0.0 {
                    continue;
                }
                if matrix[c - 1][r] == 0.0 {
                    for x in 0..size {
                        let temp: f32 = matrix[c][x];
                        matrix[c][x] = matrix[c - 1][x];
                        matrix[c - 1][x] = temp;
                    }
                    sign = -sign;
                    continue;
                }
                let req_ratio: f32 = matrix[c][r] / matrix[c - 1][r];
                for k in 0..size {
                    matrix[c][k] -= req_ratio * matrix[c - 1][k];
                }
            }
        }
        (matrix, sign)
    }

    /// Returns echelon form of initial matrix.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<4, 4> = Matrix::from([
    ///     [2.0, 3.0, 3.0, 1.0],
    ///     [0.0, 4.0, 3.0, -3.0],
    ///     [2.0, -1.0, -1.0, -3.0],
    ///     [0.0, -4.0, -3.0, 2.0]
    /// ]);
    /// let ef: Matrix<4, 4> = matrix.echelon_form();
    /// assert_eq!(ef.as_array(),
    ///     [
    ///         [2.0, 3.0, 3.0, 1.0],
    ///         [0.0, -4.0, -4.0, -4.0],
    ///         [0.0, 0.0, -1.0, -7.0],
    ///         [0.0, 0.0, 0.0, -1.0]
    ///     ]
    /// );
    /// ```
    ///
    pub fn echelon_form(&self) -> Matrix<N, N> {
        self.internal_echelon_form().0
    }

    /// Makes n-sized identity matrix.
    ///
    /// Constructs identity matrix (square matrix with 1.0 on main diagonal
    /// and 0.0 elsewhere).
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 3> = Matrix::identity();
    /// assert_eq!(
    ///     matrix.as_array(),
    ///     [
    ///         [1.0, 0.0, 0.0],
    ///         [0.0, 1.0, 0.0],
    ///         [0.0, 0.0, 1.0]
    ///     ],
    /// );
    /// ```
    ///
    pub fn identity() -> Matrix<N, N> {
        let mut matrix: Matrix<N, N> = Matrix::zero();
        for i in 0..N {
            matrix[i][i] = 1.0;
        }
        matrix
    }

    /// Returns determinant of initial matrix.
    ///
    /// Calculates determinant of a square matrix using echelon form of initial matrix. Product of
    /// its diagonal and the sign is equal to determinant.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 3> = Matrix::from([
    ///     [1.0, 2.0, 3.0],
    ///     [4.0, 5.0, 6.0],
    ///     [7.0, 8.0, 9.0]
    /// ]);
    /// assert_eq!(matrix.determinant(), 0.0);
    /// ```
    ///
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 3> = Matrix::from([
    ///     [-3.0, 2.0, 2.0],
    ///     [43.0, 1.0, -12.0],
    ///     [5.0, 0.0, 5.0]
    /// ]);
    /// assert_eq!(matrix.determinant(), -575.0);
    /// ```
    ///
    pub fn determinant(&self) -> f32 {
        if N == 0 {
            return 0.0;
        }
        let (ef, sign): (Matrix<N, N>, Sign) = self.internal_echelon_form();
        let mut product: f32 = 1.0;
        for i in 0..N {
            product *= ef[i][i];
        }
        product * f32::from(sign as i8)
    }
    /// Returns inverse of an initial matrix
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let matrix: Matrix<3, 3> = Matrix::from([
    ///     [3.0, 2.0, 2.0],
    ///     [1.0, 2.0, 2.0],
    ///     [1.0, 3.0, 2.0]
    /// ]);
    /// let mut inverse: Matrix<3, 3> = matrix
    ///     .inverse()
    ///     .expect("Should not fail: determinant is not equal to zero.").round_up_to(2);
    /// assert_eq!(
    ///     inverse.as_array(),
    ///     [
    ///         [0.5, -0.5, 0.0],
    ///         [0.0, -1.0, 1.0],
    ///         [-0.25, 1.75, -1.0]
    ///     ]
    /// );
    /// ```
    ///
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// let matrix: Matrix<3, 3> = Matrix::from([
    ///     [1.0, 2.0, 3.0],
    ///     [4.0, 5.0, 6.0],
    ///     [7.0, 8.0, 9.0]
    /// ]);
    /// assert!(matrix.inverse().is_none());
    /// ```
    ///
    pub fn inverse(&self) -> Option<Matrix<N, N>> {
        if self.determinant() == 0.0 {
            return None;
        }
        let (m, mut i): (Matrix<N, N>, Matrix<N, N>) = (*self, Matrix::identity());
        let mut vec: Vec<Vec<f32>> = vec![vec![0.0; N * 2]; N];
        for (r, row) in vec.iter_mut().enumerate().take(N) {
            row[..N].copy_from_slice(&m[r][..N]);
            row[N..(N + N)].copy_from_slice(&i[r][..N]);
        }
        let rref: Vec<Vec<f32>> = Matrix::<N, N>::internal_rref(vec);
        for (r, row) in rref.iter().enumerate().take(N) {
            i[r][..N].copy_from_slice(&row[N..(N + N)]);
        }
        Some(i)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> FloatOperations for Matrix<ROWS, COLUMNS> {
    /// Constructs new matrix by correcting every matrix element that may be wronged by float operations.
    ///
    /// Fixes such things as -0.0 into 0.0, 0.00000001 into 0.0 and 0.99999999 into 1.0.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let mut matrix: Matrix<1, 3> = Matrix::from([[-0.0, 0.00000001, 0.99999999]]).correct(0);
    /// assert_eq!(matrix.as_array(), [[0.0, 0.0, 1.0]]);
    /// ```
    ///
    fn correct(self, digits: i32) -> Self {
        self.map(|elem| elem.correct(digits))
    }
    /// Constructs new matrix by rounding every matrix element up to specified number of digits after floating
    /// point.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::Matrix;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let mut matrix: Matrix<1, 3> = Matrix::from([[0.015, 0.00005, 0.1]]).round_up_to(2);
    /// assert_eq!(matrix.as_array(), [[0.02, 0.00, 0.10]]);
    /// ```
    ///
    fn round_up_to(self, digits: i32) -> Self {
        self.map(|elem| elem.round_up_to(digits))
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Index<usize> for Matrix<ROWS, COLUMNS> {
    type Output = [f32; COLUMNS];

    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index]
    }
}
impl<const ROWS: usize, const COLUMNS: usize> IndexMut<usize> for Matrix<ROWS, COLUMNS> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.arr[index]
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Neg for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns negated matrix.
    ///
    /// Is equal to `self.map(|x| -x)` and `self * -1.0`.
    ///
    fn neg(self) -> Self::Output {
        self.map(|x| -x)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Add<Self> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where each element is a sum of those elements in given
    /// matrices.
    ///
    /// Is equal to `self.combine(rhs, |a, b| a + b)`. \
    ///
    fn add(self, rhs: Self) -> Self::Output {
        self.combine(rhs, |a, b| a + b)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Sub<Self> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where each element is a difference of those elements in given
    /// matrices.
    ///
    /// Is equal to `self.combine(rhs, |a, b| a - b)`.
    ///
    fn sub(self, rhs: Self) -> Self::Output {
        self.combine(rhs, |a, b| a - b)
    }
}
impl<const ROWS: usize, const COLUMNS: usize, const RHS_COLUMNS: usize>
    Mul<Matrix<COLUMNS, RHS_COLUMNS>> for Matrix<ROWS, COLUMNS>
{
    type Output = Matrix<ROWS, RHS_COLUMNS>;

    /// Performs dot product operation on two matrices.
    ///
    /// Is equal to `self.dot(rhs)`
    ///
    fn mul(self, rhs: Matrix<COLUMNS, RHS_COLUMNS>) -> Self::Output {
        self.dot_product(rhs)
    }
}

impl<const ROWS: usize, const COLUMNS: usize> AddAssign<Self> for Matrix<ROWS, COLUMNS> {
    /// Adds corresponding element of rhs matrix to each element in initial matrix.
    ///
    /// Is equal to `*self = self.combine(rhs, |a, b| a + b)`.
    ///
    fn add_assign(&mut self, rhs: Self) {
        *self = self.combine(rhs, |a, b| a + b);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> SubAssign<Self> for Matrix<ROWS, COLUMNS> {
    /// Subtracts corresponding element of rhs matrix from each element in initial matrix.
    ///
    /// Is equal to `*self = self.combine(rhs, |a, b| a - b)`.
    ///
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.combine(rhs, |a, b| a - b);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Add<f32> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where given value is added to each element.
    ///
    /// Is equal to `self.map(|x| x + rhs)`.
    ///
    fn add(self, rhs: f32) -> Self::Output {
        self.map(|x| x + rhs)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Sub<f32> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where given value is subtracted from each element.
    ///
    /// Is equal to `self.map(|x| x - rhs)`.
    ///
    fn sub(self, rhs: f32) -> Self::Output {
        self.map(|x| x - rhs)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Mul<f32> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where each element is multiplied by given multiplier.
    ///
    /// Is equal to `self.map(|x| x * rhs)`.
    ///
    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|x| x * rhs)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Div<f32> for Matrix<ROWS, COLUMNS> {
    type Output = Self;

    /// Returns from matrix where each element is divided by given value.
    ///
    /// Is equal to `self.map(|x| x / rhs)`.
    ///
    fn div(self, rhs: f32) -> Self::Output {
        self.map(|x| x / rhs)
    }
}
impl<const ROWS: usize, const COLUMNS: usize> AddAssign<f32> for Matrix<ROWS, COLUMNS> {
    /// Adds given value to every matrix element.
    ///
    /// Is equal to `*self = self.map(|x| x + rhs)`.
    ///
    fn add_assign(&mut self, rhs: f32) {
        *self = self.map(|x| x + rhs);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> SubAssign<f32> for Matrix<ROWS, COLUMNS> {
    /// Subtracts given value from every matrix element.
    ///
    /// Is equal to `*self = self.map(|x| x - rhs)`.
    ///
    fn sub_assign(&mut self, rhs: f32) {
        *self = self.map(|x| x - rhs);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> MulAssign<f32> for Matrix<ROWS, COLUMNS> {
    /// Multiplies each matrix element by given multiplier.
    ///
    /// Is equal to `*self = self.map(|x| x * rhs))`.
    ///
    fn mul_assign(&mut self, rhs: f32) {
        *self = self.map(|x| x * rhs);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> DivAssign<f32> for Matrix<ROWS, COLUMNS> {
    /// Divides every matrix element by given value.
    ///
    /// Is equal to `*self = self.map(|x| x / rhs)`.
    ///
    fn div_assign(&mut self, rhs: f32) {
        *self = self.map(|x| x / rhs);
    }
}
impl<const ROWS: usize, const COLUMNS: usize> PartialEq for Matrix<ROWS, COLUMNS> {
    /// Checks if matrices are equal.
    ///
    fn eq(&self, other: &Self) -> bool {
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                if !equal(self.arr[r][c], other.arr[r][c]) {
                    return false;
                }
            }
        }
        true
    }
}
impl<const ROWS: usize, const COLUMNS: usize> Eq for Matrix<ROWS, COLUMNS> {}
impl<const ROWS: usize, const COLUMNS: usize> From<[[f32; COLUMNS]; ROWS]>
    for Matrix<ROWS, COLUMNS>
{
    /// Shorthand for writing `Matrix { arr: ... }`.
    ///
    fn from(arr: [[f32; COLUMNS]; ROWS]) -> Self {
        let mut array: Array<Array<f32, COLUMNS>, ROWS> = Array([Array([0.0; COLUMNS]); ROWS]);
        for r in 0..ROWS {
            array[r] = Array(arr[r]);
        }
        Matrix { arr: array }
    }
}

/// Type alias for 3x1 [`Matrix`] (is used to represent two-dimensional vector).
///
pub type Matrix3x1 = Matrix<3, 1>;
impl From<Vector2> for Matrix3x1 {
    /// `From<Vector2>` trait for `Matrix3x1` can be used in transforming.
    ///
    /// Resulting matrix is corrected.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::{Matrix3x3, Matrix3x1};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// let vector: Vector2 = Vector2::from([0.0, 2.0]);
    /// let rotation_matrix: Matrix3x3 = Matrix3x3::from([
    ///     [0.66, -0.75, 0.0],
    ///     [0.75, 0.66, 0.0],
    ///     [0.0, 0.0, 1.0]
    /// ]);
    /// assert_eq!(
    ///     (rotation_matrix * Matrix3x1::from(vector)).as_array(),
    ///     [
    ///         [-1.5],
    ///         [1.32],
    ///         [1.0]
    ///     ]
    /// );
    /// ```
    ///
    fn from(vector: Vector2) -> Self {
        Matrix::from([[vector.x], [vector.y], [1.0]])
    }
}
impl From<Matrix3x1> for Vector2 {
    /// `From<Matrix3x1>` trait for `Vector2` can be used in transforming.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::matrices::{Matrix3x3, Matrix3x1};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// let vector: Vector2 = Vector2::from([0.0, 2.0]);
    /// let rotation_matrix: Matrix3x3 = Matrix3x3::from([
    ///     [0.66, -0.75, 0.0],
    ///     [0.75, 0.66, 0.0],
    ///     [0.0, 0.0, 1.0]
    /// ]);
    /// let res: Vector2 = Vector2::from(rotation_matrix * Matrix3x1::from(vector));
    /// assert_eq!(res, Vector2::from([-1.5, 1.32]));
    /// ```
    ///
    fn from(matrix: Matrix3x1) -> Self {
        Vector2::from([matrix[0][0], matrix[1][0]])
    }
}
/// Type alias for 3x3 [`Matrix`] (two-dimensional transform matrix).
///
pub type Matrix3x3 = Matrix<3, 3>;
impl Matrix3x3 {
    /// Transforms given vector by using dot product (shorthand for writing `Vector2::from(self * Matrix3x1::from(vector))`).
    ///
    pub fn apply_to(self, vector: Vector2) -> Vector2 {
        Vector2::from(self * Matrix3x1::from(vector))
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn matrix() {
        let m1: Matrix<1, 3> = Matrix::from([[1.0, 2.0, 3.0]]);
        assert_eq!(m1[0][1], 2.0);

        let m2: Matrix<1, 3> = Matrix::from([[3.0, 2.0, 1.0]]);
        let mut m3: Matrix<1, 3> = m1;

        assert_eq!((m1 + m2).as_array(), [[4.0; 3]]);
        assert_eq!((m1 - m2).as_array(), [[-2.0, 0.0, 2.0]]);
        assert_eq!((m1 * m2.transpose()).as_array(), [[10.0]]);

        m3 += m2;
        assert_eq!(m3.as_array(), [[4.0; 3]]);
        m3 -= m2;
        assert_eq!(m3.as_array(), [[1.0, 2.0, 3.0]]);

        assert_eq!((m1 + 2.0).as_array(), [[3.0, 4.0, 5.0]]);
        assert_eq!((m1 - 2.0).as_array(), [[-1.0, 0.0, 1.0]]);
        assert_eq!((m1 * 2.0).as_array(), [[2.0, 4.0, 6.0]]);
        assert_eq!((m1 / 2.0).as_array(), [[0.5, 1.0, 1.5]]);

        m3 += 2.0;
        assert_eq!(m3.as_array(), [[3.0, 4.0, 5.0]]);
        m3 -= 2.0;
        assert_eq!(m3.as_array(), [[1.0, 2.0, 3.0]]);
        m3 *= 2.0;
        assert_eq!(m3.as_array(), [[2.0, 4.0, 6.0]]);
        m3 /= 2.0;
        assert_eq!(m3.as_array(), [[1.0, 2.0, 3.0]]);
    }
}
