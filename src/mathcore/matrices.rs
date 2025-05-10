//! `mathcore::matrices` submodule implements NxM matrices which can be used to apply transformations
//! on vectors.
//!

use crate::mathcore::{
    floats::{almost_equal, FloatOperations},
    vectors::Vector2,
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
        let mut arr = [[0.0; COLUMNS]; ROWS];
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
        let mut matrix = Matrix::zero();
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
        let mut matrix = Matrix::zero();
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
        let mut matrix = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                matrix[c][r] = self[r][c];
            }
        }
        matrix
    }

    /// `internal_row_reduced_echelon_form` operates on `Vec<f32>` which represents two-dimensional array.
    ///
    fn internal_row_reduced_echelon_form(
        matrix: &mut Vec<f32>,
        rows: usize,
        columns: usize,
    ) -> f32 {
        if matrix.is_empty() {
            return 0.0;
        }

        let index = |r, c| c + r * columns;

        if matrix[index(0, 0)] == 0.0 {
            let mut row_i = 0;
            for r in 0..rows {
                if matrix[index(r, 0)] > 0.0 {
                    row_i = r;
                    break;
                }
            }
            for c in 0..columns {
                (matrix[index(row_i, c)], matrix[index(0, c)]) =
                    (matrix[index(0, c)], matrix[index(row_i, c)]);
            }
        }

        let mut carry = 1.0;
        for lead in 0..rows {
            let leader = matrix[index(lead, lead)];
            if leader == 0.0 {
                carry = 0.0;
                continue;
            }

            carry *= leader;
            for c in 0..columns {
                matrix[index(lead, c)] /= leader;
            }
            for r in 0..rows {
                if r == lead {
                    continue;
                }

                let k = matrix[index(r, lead)];
                dbg!(r, lead, leader, matrix[index(r, lead)], k);
                for c in 0..columns {
                    dbg!(matrix[index(r, c)], matrix[index(lead, c)]);
                    matrix[index(r, c)] -= matrix[index(lead, c)] * k;
                }
                dbg!(&matrix);
            }
        }
        carry
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
    ///     matrix.row_reduced_echelon_form().correct_to(0).as_array(),
    ///     [
    ///         [1.0, 0.0, 0.0, 2.0],
    ///         [0.0, 1.0, 0.0, 4.0],
    ///         [0.0, 0.0, 1.0, -3.0]
    ///     ],
    /// );
    /// ```
    ///
    pub fn row_reduced_echelon_form(&self) -> Matrix<ROWS, COLUMNS> {
        let mut m = vec![];
        for row in self.arr.0 {
            m.extend(row.0);
        }
        let _ = Self::internal_row_reduced_echelon_form(&mut m, ROWS, COLUMNS);

        let mut matrix = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..COLUMNS {
                matrix[r][c] = m[c + r * COLUMNS];
            }
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
    /// assert_eq!(m1.matmul(m2).as_array(), [[14.0]]);
    /// ```
    ///
    pub fn matmul<const RHS_COLUMNS: usize>(
        self,
        other: Matrix<COLUMNS, RHS_COLUMNS>,
    ) -> Matrix<ROWS, RHS_COLUMNS> {
        let mut matrix = Matrix::zero();
        for r in 0..ROWS {
            for c in 0..RHS_COLUMNS {
                let mut res = 0.0;
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
        let mut matrix = Matrix::zero();
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
        let mut m = vec![];
        for row in self.arr.0 {
            m.extend(row.0);
        }

        Self::internal_row_reduced_echelon_form(&mut m, N, N)
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
    ///     .expect("Determinant is not equal to zero.").round_up_to(2);
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
        let mut identity = Matrix::identity();

        let mut m: Vec<f32> = vec![];
        for r in 0..N {
            m.extend(self.arr.0[r].0);
            m.extend(identity.arr.0[r].0);
        }

        let carry = Self::internal_row_reduced_echelon_form(&mut m, N, N * 2);
        if carry == 0.0 {
            return None;
        }
        for r in 0..N {
            for c in 0..N {
                identity[r][c] = m[(N + c) + r * (2 * N)]
            }
        }
        Some(identity)
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
    /// let mut matrix: Matrix<1, 3> = Matrix::from([[-0.0, 0.00000001, 0.99999999]]).correct_to(0);
    /// assert_eq!(matrix.as_array(), [[0.0, 0.0, 1.0]]);
    /// ```
    ///
    fn correct_to(self, digits: i32) -> Self {
        self.map(|elem| elem.correct_to(digits))
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
        self.matmul(rhs)
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
                if !almost_equal(self.arr[r][c], other.arr[r][c]) {
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
        let mut array = Array([Array([0.0; COLUMNS]); ROWS]);
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
    /// let vector: Vector2 = Vector2 { x: 0.0, y: 2.0 };
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
    /// let vector: Vector2 = Vector2 { x: 0.0, y: 2.0 };
    /// let rotation_matrix: Matrix3x3 = Matrix3x3::from([
    ///     [0.66, -0.75, 0.0],
    ///     [0.75, 0.66, 0.0],
    ///     [0.0, 0.0, 1.0]
    /// ]);
    /// let res: Vector2 = Vector2::from(rotation_matrix * Matrix3x1::from(vector));
    /// assert_eq!(res, Vector2 { x: -1.5, y: 1.32 });
    /// ```
    ///
    fn from(matrix: Matrix3x1) -> Self {
        Vector2 {
            x: matrix[0][0],
            y: matrix[1][0],
        }
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
        let m1 = Matrix::from([[1.0, 2.0, 3.0]]);
        assert_eq!(m1[0][1], 2.0);

        let m2 = Matrix::from([[3.0, 2.0, 1.0]]);
        let mut m3 = m1;

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
