def main():
    a = 0
    b = 1
    for _ in range(10000000):
        tmp = a + b
        a = b
        b = tmp
    print(b)

if __name__ == "__main__":
    main()
