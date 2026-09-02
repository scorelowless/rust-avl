#include <stdint.h>
#include <stdio.h>
#include "avl_lib.h"

int main() {
    AVLTree* tree = avl_create();

    avl_insert(tree, 1, "one");
    avl_insert(tree, 2, "two");
    avl_insert(tree, 3, "three");

    if (avl_contains(tree, 2)) {
        const char* value = avl_get(tree, 2);
        printf("Key 2 has value: %s\n", value);
        free_string(value);
    }

    const char* value = avl_get(tree, 4);
    if (value == NULL) {
        printf("Key 4 not found in the tree\n");
    }

    const char* another_value = avl_get(tree, 1);
    if (another_value != NULL) {
        printf("Key 1 has value: %s\n", another_value);
        free_string(another_value);
    }

    avl_delete(tree, 2);

    if (!avl_contains(tree, 2)) {
        printf("Key 2 has been deleted\n");
    }

    avl_free(tree);
    return 0;
}