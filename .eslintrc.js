module.exports = {
    "env": {
        "browser": true,
        "es6": true
    },
    "extends": "eslint:recommended",
    "parserOptions": {
        "ecmaVersion": 2018
    },
    "rules": {
        "indent": [
            "error",
            2,
            {
                "SwitchCase": 1,
            },
        ],
        "linebreak-style": ["error", "unix"],
        "quotes": ["error", "single"],
        "semi": ["error", "always"],
        "eqeqeq": ["error", "always"],
        "dot-location": ["error", "property"],
        "dot-notation": ["warn"],
        "no-else-return": ["warn"],
        "no-var": ["error"],
        "arrow-body-style": ["warn"],
        "object-shorthand": ["warn"],
        "prefer-arrow-callback": ["warn"],
        "prefer-const": ["warn"],
        "prefer-template": ["warn"],
        "comma-dangle": ["error", "always-multiline"],
        "array-bracket-newline": ["warn", "consistent"],
        "array-bracket-spacing": ["warn", "never"],
        "block-spacing": ["warn", "always"],
        "brace-style": ["warn", "1tbs"],
        "camelcase": ["warn"],
        "no-extra-parens": ["warn"],
        "no-template-curly-in-string": ["warn"],
        "valid-jsdoc": [
            "error",
            {
                "requireReturn": false,
            },
        ],
        "array-callback-return": ["error"],
        "block-scoped-var": ["error"],
        "consistent-return": ["error"],
        "curly": ["warn"],
        "no-implicit-globals": ["error"],
        "no-useless-concat": ["warn"],
        "radix": ["error", "always"],
    }
};
