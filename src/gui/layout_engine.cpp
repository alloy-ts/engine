#include <vector>
#include <string>

enum class LayoutDirection { HORIZONTAL, VERTICAL };
enum class JustifyContent { START, CENTER, END, SPACE_BETWEEN };
enum class AlignItems { START, CENTER, END, STRETCH };

struct Rect {
    int x, y, width, height;
};

struct LayoutNode {
    int control_id;
    LayoutDirection direction = LayoutDirection::VERTICAL;
    float flex = 0.0f;
    int gap = 0, padding_top = 0, padding_right = 0, padding_bottom = 0, padding_left = 0;
    JustifyContent justify_content = JustifyContent::START;
    AlignItems align_items = AlignItems::STRETCH;
    int fixed_width = 0, fixed_height = 0;
    std::vector<LayoutNode*> children;
    Rect frame = {0, 0, 0, 0};
};

class LayoutEngine {
public:
    void compute(LayoutNode* root, int width, int height) {
        if (!root) return;
        root->frame = {0, 0, width, height};
        compute_recursive(root);
    }

private:
    void compute_recursive(LayoutNode* node) {
        /* Minimal Flex-like implementation placeholder */
        int current_x = node->frame.x + node->padding_left;
        int current_y = node->frame.y + node->padding_top;

        for (auto* child : node->children) {
            child->frame.x = current_x;
            child->frame.y = current_y;
            /* Placeholder: give each child some fixed size for now */
            child->frame.width = 100;
            child->frame.height = 30;

            if (node->direction == LayoutDirection::VERTICAL) {
                current_y += child->frame.height + node->gap;
            } else {
                current_x += child->frame.width + node->gap;
            }
            compute_recursive(child);
        }
    }
};
