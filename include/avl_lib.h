#ifndef RUST_CODE_AVL_LIB_H
#define RUST_CODE_AVL_LIB_H

typedef struct AVLTree AVLTree;

AVLTree* avl_create();
void avl_free(AVLTree* tree);
int32_t avl_insert(AVLTree* tree, uint64_t key, const char* value);
int32_t avl_contains(AVLTree* tree, uint64_t key);
const char* avl_get(AVLTree* tree, uint64_t key);
void free_string(const char* str);
int32_t avl_delete(AVLTree* tree, uint64_t key);

#endif //RUST_CODE_AVL_LIB_H