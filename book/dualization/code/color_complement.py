import colorsys


class ColorComplement:
    """Calculate color complements for hex colors only"""

    @staticmethod
    def hex_to_rgb(hex_color):
        """Convert hex color code to RGB tuple (0-255)"""
        hex_color = hex_color.lstrip("#")
        return tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))

    @staticmethod
    def rgb_to_hex(r, g, b):
        """Convert RGB tuple to hex color code"""
        return f"#{r:02x}{g:02x}{b:02x}"

    @staticmethod
    def hex_complement(hex_color, method="hsv"):
        """
        Get complement of a hex color

        Args:
            hex_color: Hex color string (e.g., '#FF0000' or 'FF0000')
            method: 'hsv' (default) or 'rgb'

        Returns:
            Hex string of complement color
        """
        # Convert hex to RGB
        r, g, b = ColorComplement.hex_to_rgb(hex_color)

        if method == "hsv":
            # Convert RGB to HSV
            h, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
            # Add 180 degrees (0.5 in normalized hue space)
            h_complement = (h + 0.5) % 1.0
            # Convert back to RGB
            r_comp, g_comp, b_comp = colorsys.hsv_to_rgb(h_complement, s, v)
            r_comp, g_comp, b_comp = (
                int(r_comp * 255),
                int(g_comp * 255),
                int(b_comp * 255),
            )
        elif method == "rgb":
            # Simple RGB complement
            r_comp, g_comp, b_comp = 255 - r, 255 - g, 255 - b
        else:
            raise ValueError("Method must be 'hsv' or 'rgb'")

        return ColorComplement.rgb_to_hex(r_comp, g_comp, b_comp)


# Example usage and testing
if __name__ == "__main__":
    # Find complement of '#0d0887' and other colors
    test_colors = ["#0d0887", "#FF0000", "#0000FF", "#800080", "#FFA500", "#008000"]

    print("Hex Color Complements:")
    print("-" * 50)
    print(f"{'Original':<12} {'HSV Complement':<15} {'RGB Complement'}")
    print("-" * 50)

    for hex_color in test_colors:
        hsv_comp = ColorComplement.hex_complement(hex_color, "hsv")
        rgb_comp = ColorComplement.hex_complement(hex_color, "rgb")
        print(f"{hex_color:<12} {hsv_comp:<15} {rgb_comp}")

    print("\nSpecific example - '#0d0887':")
    print("-" * 30)
    target = "#0d0887"
    complement = ColorComplement.hex_complement(target)
    print(f"Original:   {target}")
    print(f"Complement: {complement}")
